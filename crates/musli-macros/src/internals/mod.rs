pub(crate) mod apply;
pub(crate) mod attr;
pub(crate) mod build;
mod ctxt;
mod expansion;
mod mode;
mod name;
mod packed;
mod tokens;

pub(crate) const ATTR: &str = "musli";

pub(crate) use self::attr::Only;
pub(crate) use self::build::Build;
pub(crate) use self::ctxt::Ctxt;
pub(crate) use self::expansion::Expansion;
pub(crate) use self::mode::{ImportedMethod, Mode};
pub(crate) use self::name::NameAll;
pub(crate) use self::packed::packed;
pub(crate) use self::tokens::{Import, Tokens};

pub(crate) type Result<T, E = ()> = std::result::Result<T, E>;
