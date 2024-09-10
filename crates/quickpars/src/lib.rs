//! QuickJS Bytecode Parser written in Rust.

use core::str;

use anyhow::{anyhow, Context, Result};

pub mod atom;
pub use atom::*;
pub mod bc;
pub use bc::*;
pub mod consts;
pub use consts::*;
pub mod js_module;
pub use js_module::*;
pub mod op;
pub use op::*;
pub mod readers;
pub use readers::*;
pub mod sections;
pub use sections::*;

/// Known payload in the bytecode.
#[derive(Debug, Clone)]
pub enum Payload<'a> {
    Version(u8),
    Header(HeaderSection),
    Module(ModuleSection),
    Function(FunctionSection<'a>),
    End,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum ParserState {
    Version,
    Header,
    Tags,
    End,
}

/// A QuickJS bytecode parser.
#[derive(Debug, Copy, Clone)]
pub struct Parser {
    /// The state of the parser.
    state: ParserState,
    /// The current position of the parser.
    offset: usize,
    /// Is the parser done.
    done: bool,
}

impl Parser {
    /// Create a new [Parser].
    pub fn new() -> Self {
        Self {
            state: ParserState::Version,
            offset: 0,
            done: false,
        }
    }
}

impl Parser {
    /// Parse the entire bytecode buffer and returns the omplete js module.
    pub fn parse_buffer_sync(data: &[u8]) -> Result<JsModule<'_>> {
        let mut header = Err(anyhow!("No header found"));
        let mut module = Err(anyhow!("No module found"));
        let mut functions = vec![];
        for payload in Parser::new().parse_buffer(data) {
            match payload? {
                Payload::Header(h) => {
                    header = Ok(h);
                }
                Payload::Module(m) => {
                    module = Ok(m);
                }
                Payload::Function(f) => {
                    functions.push(f);
                }
                _ => {}
            }
        }
        Ok(JsModule::new(header?, module?, functions))
    }

    /// Parse the entire bytecode buffer.
    pub fn parse_buffer(self, data: &[u8]) -> impl Iterator<Item = Result<Payload<'_>>> {
        let mut parser = self;
        std::iter::from_fn(move || {
            if parser.done {
                return None;
            }
            Some(parser.parse(data))
        })
    }

    fn parse<'a>(&mut self, data: &'a [u8]) -> Result<Payload<'a>> {
        // Every time `parse` is called, make sure to update the view of data
        // that we're parsing via `&data[self.offset...]`
        let mut reader = BinaryReader::new(&data[self.offset..]);
        match self.parse_with(&mut reader) {
            Ok(payload) => {
                self.offset += reader.offset;
                if self.offset >= data.len() {
                    self.done = true;
                }
                Ok(payload)
            }
            Err(err) => {
                self.done = true;
                Err(err).with_context(|| {
                    format!(
                        "Failed to parse bytecode at offset: {} and state: {:?}",
                        self.offset + reader.offset,
                        self.state,
                    )
                })
            }
        }
    }

    fn parse_with<'a: 'b, 'b>(&mut self, reader: &'b mut BinaryReader<'a>) -> Result<Payload<'a>> {
        use Payload::*;

        let data = reader.data();
        match self.state {
            ParserState::Version => reader
                .read_u8()
                .and_then(validate_version)
                .map(Version)
                .map(|v| {
                    self.state = ParserState::Header;
                    v
                }),

            ParserState::Header => {
                let atom_count = reader.read_leb128()?;
                self.compute_atoms_with_size(atom_count, &data[reader.offset..])
                    .and_then(|(atoms, size)| {
                        reader.skip(size)?;
                        Ok(atoms)
                    })
                    .map(|atoms| {
                        self.state = ParserState::Tags;
                        Header(HeaderSection::new(atom_count, atoms))
                    })
            }
            ParserState::Tags => reader
                .read_u8()
                .and_then(Tag::map_byte)
                .and_then(|tag| self.parse_tag(tag, reader)),
            ParserState::End => {
                self.done = true;
                Ok(End)
            }
        }
    }

    fn parse_tag<'a: 'b, 'b>(
        &mut self,
        tag: Tag,
        reader: &'b mut BinaryReader<'a>,
    ) -> Result<Payload<'a>> {
        use Payload::*;

        let data = reader.data();
        let payload = match tag {
            Tag::Module => {
                // The index of the atom array containing the module name.
                let name_index = reader.read_atom()?;
                let (module, size) =
                    self.compute_module_section_with_size(&data[reader.offset..], name_index)?;
                reader.skip(size)?;
                Ok(Module(module))
            }
            Tag::FunctionBytecode => {
                let flags = reader.read_u16()?;
                // JS mode.
                // Are we in `strict` mode?.
                reader.read_u8()?;
                let name_index = reader.read_atom()?;
                let arg_count = reader.read_leb128()?;
                let var_count = reader.read_leb128()?;
                let defined_arg_count = reader.read_leb128()?;
                let stack_size = reader.read_leb128()?;
                let closure_count = reader.read_leb128()?;
                let constant_pool_size = reader.read_leb128()?;
                let bytecode_len = reader.read_leb128()?;
                let local_count = reader.read_leb128()?;

                let header = FunctionSectionHeader {
                    flags,
                    name_index,
                    arg_count,
                    var_count,
                    defined_arg_count,
                    stack_size,
                    closure_count,
                    constant_pool_size,
                    bytecode_len,
                    local_count,
                };

                let debug = flag::<bool>(flags as u32, 9);

                let (locals, locals_reader) = self
                    .compute_locals_with_size(local_count, &data[reader.offset..])
                    .and_then(|(locals, size)| Ok((locals, slice(reader, size)?)))?;
                let (closures, closures_reader) = self
                    .compute_closure_with_size(closure_count, &data[reader.offset..])
                    .and_then(|(closures, size)| Ok((closures, slice(reader, size)?)))?;
                let operators_reader = slice(reader, bytecode_len as usize)?;

                let debug_info = if debug != 0 {
                    let filename = reader.read_atom()?;
                    let lineno = reader.read_leb128()?;
                    let line_debug_len = reader.read_leb128()?;
                    let line_buffer = slice(reader, line_debug_len as usize)?;

                    let colno = reader.read_leb128()?;
                    let col_debug_len = reader.read_leb128()?;
                    let col_buffer = slice(reader, col_debug_len as usize)?;
                    Some(DebugInfo::new(
                        filename,
                        lineno,
                        colno,
                        line_buffer,
                        col_buffer,
                    ))
                } else {
                    None
                };

                Ok(Function(FunctionSection::new(
                    header,
                    locals,
                    locals_reader,
                    closures,
                    closures_reader,
                    operators_reader,
                    debug_info,
                )))
            }
            x => Err(anyhow!("Unsupported {x:?}")),
        };
        if reader.done() {
            self.state = ParserState::End;
        }
        payload
    }

    /// Returns the interned atoms, and the size of the data consumed.
    fn compute_atoms_with_size(
        &self,
        atom_count: u32,
        data: &[u8],
    ) -> Result<(Vec<String>, usize)> {
        // Create a fresh new reader to make sure that we have the right information
        // regarding the bytes consumed.
        let mut reader = BinaryReader::new(data);
        let mut atoms = ATOM_NAMES
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        (0..atom_count)
            .try_for_each(|_| {
                atoms.push(str::from_utf8(read_str_bytes(&mut reader)?)?.to_string());
                Ok(())
            })
            .map(|_| (atoms, reader.offset))
    }

    /// Returns the content of the module section.
    fn compute_module_section_with_size(
        &self,
        data: &[u8],
        name_index: u32,
    ) -> Result<(ModuleSection, usize)> {
        let mut reader = BinaryReader::new(data);
        let mut req_modules = vec![];
        let mut exports = vec![];
        let mut star_exports = vec![];
        let mut imports = vec![];
        let has_tla = reader
            // Req module entries count.
            .read_leb128()
            // Each dependency.
            .and_then(|deps| {
                (0..deps).try_for_each(|_| {
                    req_modules.push(reader.read_atom()?);
                    Ok(())
                })
            })
            // Exports count.
            .and_then(|_| reader.read_leb128())
            // Each export.
            .and_then(|count| {
                (0..count).try_for_each(|_| {
                    let export_type = reader.read_u8()?;
                    if export_type == JS_EXPORT_TYPE_LOCAL {
                        // The local index of the export.
                        let var_idx = reader.read_leb128()?;
                        let export_name_idx = reader.read_atom()?;
                        exports.push(ModuleExportEntry::Local {
                            var_idx,
                            export_name_idx,
                        });
                    } else {
                        let module_idx = reader.read_leb128()?;
                        let local_name_idx = reader.read_atom()?;
                        let export_name_idx = reader.read_atom()?;
                        exports.push(ModuleExportEntry::Indirect {
                            module_idx,
                            local_name_idx,
                            export_name_idx,
                        });
                    }
                    Ok(())
                })
            })
            // Star exports count.
            .and_then(|_| reader.read_leb128())
            // Each * export
            .and_then(|count| {
                (0..count).try_for_each(|_| {
                    star_exports.push(reader.read_leb128()?);
                    Ok(())
                })
            })
            // Imports count.
            .and_then(|_| reader.read_leb128())
            // Each import.
            .and_then(|imports_count| {
                (0..imports_count).try_for_each(|_| {
                    // Variable index.
                    let var_idx = reader.read_leb128()?;
                    // Import name index;
                    let name_idx = reader.read_atom()?;
                    // Required module index.
                    let req_module_idx = reader.read_leb128()?;
                    imports.push(ModuleImportEntry {
                        var_idx,
                        name_idx,
                        req_module_idx,
                    });
                    Ok(())
                })
            })
            // has_tla
            .and_then(|_| reader.read_u8())?;
        Ok((
            ModuleSection::new(
                name_index,
                req_modules,
                exports,
                star_exports,
                imports,
                has_tla,
            ),
            reader.offset,
        ))
    }

    fn compute_locals_with_size(
        &self,
        locals_count: u32,
        data: &[u8],
    ) -> Result<(Vec<FunctionLocal>, usize)> {
        let mut reader = BinaryReader::new(data);
        let mut locals = vec![];
        (0..locals_count)
            .try_for_each(|_| {
                locals.push(FunctionLocal {
                    name_index: reader.read_atom()?,
                    scope_level: reader.read_leb128()?,
                    scope_next: reader.read_leb128()?,
                    flags: reader.read_u8()?,
                });
                Ok(())
            })
            .map(|_| (locals, reader.offset))
    }

    fn compute_closure_with_size(
        &self,
        closure_count: u32,
        data: &[u8],
    ) -> Result<(Vec<FunctionClosure>, usize)> {
        let mut reader = BinaryReader::new(data);
        let mut closures = vec![];
        (0..closure_count)
            .try_for_each(|_| {
                closures.push(FunctionClosure {
                    name_index: reader.read_atom()?,
                    index: reader.read_leb128()?,
                    flags: reader.read_u8()?,
                });
                Ok(())
            })
            .map(|_| (closures, reader.offset))
    }
}
