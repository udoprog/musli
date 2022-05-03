use std::fmt;
use std::ops::Deref;
use syn::{Ident, Path};

#[derive(Copy, Clone)]
pub struct Symbol(&'static str);

pub const ATTR: Symbol = Symbol("musli");
pub const CRATE: Symbol = Symbol("crate");
pub const WITH: Symbol = Symbol("with");
pub const DEFAULT_FIELD_NAME: Symbol = Symbol("default_field_name");
pub const DEFAULT_VARIANT_NAME: Symbol = Symbol("default_variant_name");
pub const TAG: Symbol = Symbol("tag");
pub const CONTENT: Symbol = Symbol("content");
pub const DEFAULT: Symbol = Symbol("default");
pub const SKIP_ENCODING_IF: Symbol = Symbol("skip_encoding_if");
pub const PACKED: Symbol = Symbol("packed");
pub const TRANSPARENT: Symbol = Symbol("transparent");
pub const NAME_TYPE: Symbol = Symbol("name_type");
pub const MODE: Symbol = Symbol("mode");
pub const RENAME: Symbol = Symbol("rename");

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

impl fmt::Display for Symbol {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl Deref for Symbol {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}
