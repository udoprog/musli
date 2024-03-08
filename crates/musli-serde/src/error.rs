use core::fmt;

#[derive(Debug)]
pub enum SerdeError {
    Captured,
    Custom(Box<str>),
}

impl fmt::Display for SerdeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error in musli-serde")
    }
}

impl serde::ser::Error for SerdeError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        SerdeError::Custom(format!("{}", msg).into())
    }
}

impl serde::de::Error for SerdeError {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        SerdeError::Custom(format!("{}", msg).into())
    }
}

impl std::error::Error for SerdeError {}
