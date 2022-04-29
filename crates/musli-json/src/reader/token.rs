use core::fmt;

/// Tokens.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Token {
    /// `,`.
    Comma = CM,
    /// `:`.
    Colon = CL,
    /// Whitespace.
    Whitespace = WS,
    /// `{`.
    OpenBrace = OB,
    /// `}`.
    CloseBrace = CB,
    /// `[`.
    OpenBracket = OA,
    /// `]`.
    CloseBracket = CA,
    /// A string.
    String = ST,
    /// A simple number.
    Number = NM,
    /// `null` literal.
    Null = NU,
    /// `true` literal.
    True = TR,
    /// `false` literal.
    False = FL,
    /// Error.
    Error = __,
    /// End-of-file.
    Eof = EF,
}

impl Token {
    /// Construct a token from a single byte.
    ///
    /// Note that this should optimize into a no-op beyond the lookup into the
    /// `MAP` table.
    #[inline]
    pub(crate) fn from_byte(b: u8) -> Token {
        match MAP[b as usize] {
            WS => Token::Whitespace,
            OA => Token::OpenBrace,
            CA => Token::CloseBrace,
            OB => Token::OpenBracket,
            CB => Token::CloseBracket,
            CM => Token::Comma,
            CL => Token::Colon,
            ST => Token::String,
            NM => Token::Number,
            NU => Token::Null,
            TR => Token::True,
            FL => Token::False,
            __ => Token::Error,
            _ => unreachable!(),
        }
    }

    /// Test if token is a string.
    #[inline]
    pub(crate) fn is_string(&self) -> bool {
        matches!(self, Token::String)
    }

    #[inline]
    pub(crate) fn is_value(&self) -> bool {
        (*self as u8) & VAL_BIT != 0
    }

    #[inline]
    pub(crate) fn is_null(&self) -> bool {
        matches!(self, Token::Null)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Comma => write!(f, "`,`"),
            Token::Colon => write!(f, "`:`"),
            Token::Whitespace => write!(f, "<whitespace>"),
            Token::OpenBrace => write!(f, "`{{`"),
            Token::CloseBrace => write!(f, "`}}`"),
            Token::OpenBracket => write!(f, "`[`"),
            Token::CloseBracket => write!(f, "`]`"),
            Token::String => write!(f, "`\"`"),
            Token::Number => write!(f, "<number>"),
            Token::Null => write!(f, "null"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Error => write!(f, "<error>"),
            Token::Eof => write!(f, "<eof>"),
        }
    }
}

const VAL_BIT: u8 = 0b0000_0001;
const CTL_BIT: u8 = 0b0000_0010;

const WS: u8 = 0b0000_0000;
const OA: u8 = 0b0001_0000 | VAL_BIT;
const CA: u8 = 0b0010_0000;
const OB: u8 = 0b0011_0000 | VAL_BIT;
const CB: u8 = 0b0100_0000;
const NM: u8 = 0b0101_0000 | VAL_BIT;
const ST: u8 = 0b0111_0000 | VAL_BIT;
const NU: u8 = 0b1000_0000 | VAL_BIT;
const TR: u8 = 0b1001_0000 | VAL_BIT;
const FL: u8 = 0b1010_0000 | VAL_BIT;
const CM: u8 = 0b0000_0000 | CTL_BIT;
const CL: u8 = 0b0001_0000 | CTL_BIT;
const EF: u8 = 0b1110_0000 | CTL_BIT;
const __: u8 = 0b1111_0000 | CTL_BIT;

static MAP: [u8; 256] = [
    //  1   2   3   4   5   6   7   8   9   a   b   c   d   e   f
    __, __, __, __, __, __, __, __, __, WS, WS, __, WS, WS, __, __, // 0
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 1
    WS, __, ST, __, __, __, __, __, __, __, __, __, CM, NM, __, __, // 2
    NM, NM, NM, NM, NM, NM, NM, NM, NM, NM, CL, __, __, __, __, __, // 3
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 4
    __, __, __, __, __, __, __, __, __, __, __, OB, __, CB, __, __, // 5
    __, __, __, __, __, __, FL, __, __, __, __, __, __, __, NU, __, // 6
    __, __, __, __, TR, __, __, __, __, __, __, OA, __, CA, __, __, // 7
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 8
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // 9
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // a
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // b
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // c
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // d
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // e
    __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, __, // f
];
