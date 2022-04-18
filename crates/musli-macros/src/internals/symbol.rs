use std::fmt::{self, Display};
use syn::{Ident, Path};

#[derive(Copy, Clone)]
pub struct Symbol(&'static str);

pub const ATTR: Symbol = Symbol("musli");
pub const WITH: Symbol = Symbol("with");
pub const FIELD: Symbol = Symbol("field");
pub const VARIANT: Symbol = Symbol("variant");
pub const TAG: Symbol = Symbol("tag");
pub const NAME: Symbol = Symbol("name");
pub const DEFAULT: Symbol = Symbol("default");
pub const SKIP_ENCODING_IF: Symbol = Symbol("skip_encoding_if");
pub const PACKED: Symbol = Symbol("packed");
pub const TRANSPARENT: Symbol = Symbol("transparent");

impl PartialEq<Symbol> for Ident {
    fn eq(&self, word: &Symbol) -> bool {
        self == word.0
    }
}

impl<'a> PartialEq<Symbol> for &'a Ident {
    fn eq(&self, word: &Symbol) -> bool {
        *self == word.0
    }
}

impl PartialEq<Symbol> for Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl<'a> PartialEq<Symbol> for &'a Path {
    fn eq(&self, word: &Symbol) -> bool {
        self.is_ident(word.0)
    }
}

impl Display for Symbol {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.0)
    }
}
