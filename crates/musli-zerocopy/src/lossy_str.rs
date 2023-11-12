use core::fmt;
use core::str;

#[repr(transparent)]
pub(crate) struct LossyStr([u8]);

impl LossyStr {
    /// Construct a new lossy string.
    pub(crate) fn new<S>(bytes: &S) -> &Self
    where
        S: ?Sized + AsRef<[u8]>,
    {
        // SAFETY: LossyStr is repr transparent over [u8].
        unsafe { &*(bytes.as_ref() as *const [u8] as *const LossyStr) }
    }
}

impl fmt::Debug for LossyStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut bytes = &self.0;

        write!(f, "\"")?;

        loop {
            let (string, replacement) = match str::from_utf8(bytes) {
                Ok(s) => (s, false),
                Err(e) => {
                    let (valid, invalid) = bytes.split_at(e.valid_up_to());
                    bytes = invalid.get(1..).unwrap_or_default();
                    (unsafe { str::from_utf8_unchecked(valid) }, true)
                }
            };

            for c in string.chars() {
                match c {
                    '\0' => write!(f, "\\0")?,
                    '\x01'..='\x08' | '\x0b' | '\x0c' | '\x0e'..='\x19' | '\x7f' => {
                        write!(f, "\\x{:02x}", c as u32)?;
                    }
                    _ => {
                        write!(f, "{}", c.escape_debug())?;
                    }
                }
            }

            if !replacement {
                break;
            }

            write!(f, "\u{FFFD}")?;
        }

        write!(f, "\"")?;
        Ok(())
    }
}
