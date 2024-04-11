pub(crate) mod apply;
pub mod attr;
pub(crate) mod build;
mod ctxt;
mod expansion;
mod mode;
pub(crate) mod tokens;

pub(crate) const ATTR: &str = "musli";

pub(crate) use self::attr::Only;
pub(crate) use self::ctxt::Ctxt;
pub(crate) use self::expansion::Expansion;
pub(crate) use self::mode::Mode;

pub(crate) type Result<T, E = ()> = std::result::Result<T, E>;
