/// Known payload in the bytecode.
pub enum Payload {
    Any,
}

/// The current position of the parser.
#[derive(Default)]
pub struct Offset(u64);

pub trait Handler {
    type Output;
    fn handle(payload: &Payload, offset: Offset) -> Self::Output;
}

/// A QuickJS bytecode parser.
#[derive(Default)]
pub struct Parser {
    /// The current position of the parser.
    offset: Offset,
}

impl Parser {
    pub fn parse_all(&mut self, handler: impl Handler) {}
}
