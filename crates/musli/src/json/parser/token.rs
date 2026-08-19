#![allow(clippy::identity_op, clippy::just_underscores_and_digits)]

use core::fmt;

/// Tokens.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Token {
    /// `,`.
    Comma = 0b0000_0000 | CTL_BIT,
    /// `:`.
    Colon = 0b0001_0000 | CTL_BIT,
    /// Whitespace.
    Whitespace = 0b0000_0000,
    /// `{`.
    OpenBrace = 0b0001_0000 | VAL_BIT,
    /// `}`.
    CloseBrace = 0b0010_0000,
    /// `[`.
    OpenBracket = 0b0011_0000 | VAL_BIT,
    /// `]`.
    CloseBracket = 0b0100_0000,
    /// A string.
    String = 0b0111_0000 | VAL_BIT,
    /// A simple number.
    Number = 0b0101_0000 | VAL_BIT,
    /// `null` literal.
    Null = 0b1000_0000 | VAL_BIT,
    /// `true` literal.
    True = 0b1001_0000 | VAL_BIT,
    /// `false` literal.
    False = 0b1010_0000 | VAL_BIT,
    /// Error.
    Error = 0b1111_0000 | CTL_BIT,
    /// End-of-file.
    Eof = 0b1110_0000 | CTL_BIT,
}

impl Token {
    /// Construct a token from a single byte.
    ///
    /// This is a single lookup into the `MAP` table, which stores tokens
    /// directly to avoid having to translate a byte into a token afterwards.
    #[inline]
    pub(crate) fn from_byte(b: u8) -> Token {
        MAP[b as usize]
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

const WS: Token = Token::Whitespace;
const OA: Token = Token::OpenBrace;
const CA: Token = Token::CloseBrace;
const OB: Token = Token::OpenBracket;
const CB: Token = Token::CloseBracket;
const NM: Token = Token::Number;
const ST: Token = Token::String;
const NU: Token = Token::Null;
const TR: Token = Token::True;
const FL: Token = Token::False;
const CM: Token = Token::Comma;
const CL: Token = Token::Colon;
const __: Token = Token::Error;

static MAP: [Token; 256] = [
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
