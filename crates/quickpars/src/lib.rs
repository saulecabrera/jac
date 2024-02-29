//! QuickJS Bytecode Parser written in Rust.

use anyhow::{anyhow, Context, Result};

mod bc;
mod readers;
mod sections;
use bc::{flag, validate_version, Tag};

use readers::{read_str_bytes, slice, BinaryReader};
use sections::{DebugInfo, FunctionSection, FunctionSectionHeader, HeaderSection, ModuleSection};

/// Known payload in the bytecode.
#[derive(Debug, Copy, Clone)]
pub enum Payload<'a> {
    Version(u8),
    Header(HeaderSection<'a>),
    Module(ModuleSection<'a>),
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
    /// Parse the entire bytecode buffer.
    pub fn parse_buffer<'a>(self, data: &'a [u8]) -> impl Iterator<Item = Result<Payload<'a>>> {
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
        let mut reader = BinaryReader::with_initial_offset(&data[self.offset..], self.offset);
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
                .and_then(|v| {
                    self.state = ParserState::Header;
                    Ok(v)
                }),

            ParserState::Header => {
                let atom_count = reader.read_leb128()?;
                self.compute_atoms_size(atom_count, &data[reader.offset..])
                    .and_then(|size| slice(reader, size))
                    .map(|section_reader| {
                        self.state = ParserState::Tags;
                        Header(HeaderSection::new(atom_count, section_reader))
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
        match tag {
            Tag::Module => {
                let name_index = reader.read_leb128()?;
                self.compute_module_section_size(&data[reader.offset..])
                    .and_then(|size| slice(reader, size))
                    .map(|section_reader| Module(ModuleSection::new(name_index, section_reader)))
            }
            Tag::FunctionBytecode => {
                let flags = reader.read_u16()?;
                // JS mode.
                // Unsure what this is for.
                reader.read_u8()?;
                let name_index = reader.read_leb128()?;
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

                let locals_reader = self
                    .compute_locals_size(local_count, &data[reader.offset..])
                    .and_then(|size| slice(reader, size))?;
                let closures_reader = self
                    .compute_closure_size(closure_count, &data[reader.offset..])
                    .and_then(|size| slice(reader, size))?;
                let operators_reader = slice(reader, bytecode_len as usize)?;

                let debug_info = if debug != 0 {
                    let filename = reader.read_leb128()?;
                    let lineno = reader.read_leb128()?;
                    let len = reader.read_leb128()?;

                    let buffer = slice(reader, len as usize)?;
                    Some(DebugInfo::new(filename, lineno, buffer))
                } else {
                    None
                };

                // Constants contain other bytecode functions.
                if constant_pool_size == 0 {
                    self.state = ParserState::End;
                }

                Ok(Function(FunctionSection::new(
                    header,
                    locals_reader,
                    closures_reader,
                    operators_reader,
                    debug_info,
                )))
            }
            x => Err(anyhow!("Unsupported {x:?}")),
        }
    }

    /// Calculates the amount of bytes used for the interned atoms.
    fn compute_atoms_size(&self, atom_count: u32, data: &[u8]) -> Result<usize> {
        // Create a fresh new reader to make sure that we have the right information
        // regarding the bytes consumed.
        let mut reader = BinaryReader::with_initial_offset(data, self.offset);
        (0..atom_count)
            .try_for_each(|_| {
                read_str_bytes(&mut reader)?;
                Ok(())
            })
            .map(|_| reader.offset)
    }

    /// Computes the amount of bytes needed to encode the module section.
    fn compute_module_section_size(&self, data: &[u8]) -> Result<usize> {
        let mut reader = BinaryReader::with_initial_offset(data, self.offset);
        reader
            // Req module entries count.
            .read_leb128()
            // Each dependency.
            .and_then(|deps| self.readn_leb128(deps as usize, &mut reader))
            // Exports count.
            .and_then(|_| reader.read_leb128())
            // Each export.
            .and_then(|exports| {
                (0..exports).try_for_each(|_| {
                    let export_type = reader.read_u8()?;
                    // FIXME: Put in a constant.
                    // 0 = local export.
                    // TODO: Verify the following if/else.
                    if export_type == 0 {
                        // The local index of the export.
                        reader.read_leb128()?;
                    } else {
                        // The index of the require module.
                        reader.read_leb128()?;
                        // The index of the name of the required module.
                        reader.read_leb128()?;
                    }
                    // The export name.
                    reader.read_leb128()?;
                    Ok(())
                })
            })
            // Star exports count.
            .and_then(|_| reader.read_leb128())
            // Each * export
            .and_then(|star_exports| self.readn_leb128(star_exports as usize, &mut reader))
            // Imports count.
            .and_then(|_| reader.read_leb128())
            // Each import.
            .and_then(|imports_count| {
                (0..imports_count).try_for_each(|_| {
                    // Variable index.
                    reader.read_leb128()?;
                    // Import name index;
                    reader.read_leb128()?;
                    // Required module index.
                    reader.read_leb128()?;

                    Ok(())
                })
            })
            // Return the reader offset.
            .map(|_| reader.offset)
    }

    fn compute_locals_size(&self, locals_count: u32, data: &[u8]) -> Result<usize> {
        let mut reader = BinaryReader::with_initial_offset(data, self.offset);
        (0..locals_count)
            .try_for_each(|_| {
                // Var name.
                reader.read_leb128()?;
                // Scope level.
                reader.read_leb128()?;
                // Scope next.
                reader.read_leb128()?;
                // Flags.
                reader.read_u8()?;
                Ok(())
            })
            .map(|_| reader.offset)
    }

    fn compute_closure_size(&self, closure_count: u32, data: &[u8]) -> Result<usize> {
        let mut reader = BinaryReader::with_initial_offset(data, self.offset);
        (0..closure_count)
            .try_for_each(|_| {
                // Var name.
                reader.read_leb128()?;
                // Index.
                reader.read_leb128()?;
                // Flags.
                reader.read_u8()?;
                Ok(())
            })
            .map(|_| reader.offset)
    }

    /// Reads `n` leb128 encodings.
    fn readn_leb128(&self, n: usize, reader: &mut BinaryReader<'_>) -> Result<()> {
        (0..n).try_for_each(|_| {
            reader.read_leb128()?;
            Ok(())
        })
    }
}
