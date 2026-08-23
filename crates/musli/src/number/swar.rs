//! Reading eight digits at a time out of a machine word.
//!
//! Read a byte at a time, a run of digits is a serial dependency: every digit
//! waits for the multiply which folded the one before it into the accumulator.
//! Eight bytes read as one word can instead be checked and folded together with
//! a handful of shifts and multiplies which the processor is free to overlap
//! with each other, so a long number costs a fraction of what it does one byte
//! at a time.
//!
//! Both routines answer `None` for a word which is not eight digits, whether
//! because the number ends inside it or because it is malformed, which is how
//! the caller learns to go back to reading a byte at a time.

/// The high bit of every byte, which is where the tests below leave their
/// answers.
const HIGH: u64 = 0x8080_8080_8080_8080;

/// Read the eight bytes of `buf` at `at`.
///
/// The caller has already established that there are eight bytes there to read.
#[inline(always)]
pub(super) fn word(buf: &[u8], at: usize) -> [u8; 8] {
    let mut word = [0; 8];
    word.copy_from_slice(&buf[at..at + 8]);
    word
}

/// Read eight decimal digits, the first of them most significant.
///
/// This is the trick from ["Fast numeric string to int"], which the decimal to
/// float conversion in the standard library also uses: every digit sits in
/// `0x30..=0x39`, so the eight of them fold together in three multiplies rather
/// than the eight a digit at a time takes.
///
/// ["Fast numeric string to int"]: https://johnnylee-sde.github.io/Fast-numeric-string-to-int/
#[inline]
pub(super) fn dec8(word: [u8; 8]) -> Option<u32> {
    // The word is read least significant byte first, which puts the first digit
    // in the low byte, since that is the order the multiplies below expect.
    let v = u64::from_le_bytes(word);

    // A byte over `9` shows up in the high bit of the first sum and one under
    // `0` in the high bit of the second, so between them one test covers all
    // eight bytes.
    let over = v.wrapping_add(0x4646_4646_4646_4646);
    let digits = v.wrapping_sub(0x3030_3030_3030_3030);

    if (over | digits) & HIGH != 0 {
        return None;
    }

    const MASK: u64 = 0x0000_00ff_0000_00ff;
    const MUL1: u64 = 0x000f_4240_0000_0064;
    const MUL2: u64 = 0x0000_2710_0000_0001;

    // Pairs of digits first, which fits in 63 bits and so cannot overflow, and
    // then the two multiplies which weigh the pairs by their power of ten and
    // sum them by carrying them into the top half of the word.
    let v = (digits * 10) + (digits >> 8);
    let v1 = (v & MASK).wrapping_mul(MUL1);
    let v2 = ((v >> 16) & MASK).wrapping_mul(MUL2);
    Some((v1.wrapping_add(v2) >> 32) as u32)
}

/// Read eight hexadecimal digits, the first of them most significant.
///
/// Hexadecimal is the easier of the two to fold, since a digit is a nibble and
/// eight of them are a `u32` exactly, so the digits only have to be moved next
/// to each other rather than weighed. What costs something here is telling a
/// digit from anything else, which takes three ranges rather than one.
#[inline]
pub(super) fn hex8(word: [u8; 8]) -> Option<u32> {
    /// Per byte, the high bit of the answer is set where the byte is at least
    /// `n`. Only exact while every byte is ASCII, which is tested alongside.
    ///
    /// Setting the high bit of every byte first is what stops a byte which
    /// borrows from reaching into the one above it.
    const fn at_least(v: u64, n: u64) -> u64 {
        const ONES: u64 = 0x0101_0101_0101_0101;
        (v | HIGH).wrapping_sub(n * ONES) & HIGH
    }

    // The word is read most significant byte first, which puts the first digit
    // in the top nibble, where the packing below wants it.
    let v = u64::from_be_bytes(word);

    // Setting the case bit folds `A..=F` onto `a..=f`, so the letters are one
    // range rather than two. It also folds `0x10..=0x19` onto `0..=9`, which is
    // what the separate test against `0` is there to rule out.
    let folded = v | 0x2020_2020_2020_2020;

    let ascii = !v & HIGH;
    let at_least_zero = at_least(v, 0x30);
    let past_nine = at_least(folded, 0x3a);
    let letter = at_least(folded, 0x61) & !at_least(folded, 0x67);

    if ascii & at_least_zero & (!past_nine | letter) & HIGH != HIGH {
        return None;
    }

    // Bit 6 is set for a letter and clear for a digit, which is exactly the 9
    // between where `a` and `1` land, so one term covers both.
    let nibbles = (v & 0x0f0f_0f0f_0f0f_0f0f) + ((v & 0x4040_4040_4040_4040) >> 6) * 9;

    // Nibbles into pairs, pairs into halves, halves into the answer.
    let v = (nibbles | (nibbles >> 4)) & 0x00ff_00ff_00ff_00ff;
    let v = (v | (v >> 8)) & 0x0000_ffff_0000_ffff;
    Some((v | (v >> 16)) as u32)
}

#[cfg(test)]
mod tests {
    use super::{dec8, hex8, word};

    /// What the word at a time routines have to agree with: the same eight
    /// bytes read one at a time.
    fn reference(word: [u8; 8], radix: u32) -> Option<u32> {
        let mut value = 0u32;

        for b in word {
            value = value * radix + (b as char).to_digit(radix)?;
        }

        Some(value)
    }

    /// Every byte value in every position, against a word which is otherwise
    /// digits, which is what decides whether a byte is one and where in the
    /// answer it lands.
    #[test]
    fn every_byte_in_every_position() {
        for at in 0..8 {
            for b in 0..=u8::MAX {
                let mut w = *b"12345678";
                w[at] = b;

                assert_eq!(dec8(w), reference(w, 10), "decimal {w:?}");
                assert_eq!(hex8(w), reference(w, 16), "hexadecimal {w:?}");

                // The same again over a word of letters, so that a byte is also
                // seen next to digits which are not in `0..=9`.
                let mut w = *b"abcdefAB";
                w[at] = b;

                assert_eq!(hex8(w), reference(w, 16), "hexadecimal {w:?}");
            }
        }
    }

    /// Whole words drawn from the bytes which are on the edge of being a digit,
    /// so that every combination of the three ranges is covered.
    #[test]
    fn words_of_edge_bytes() {
        const EDGES: &[u8] = &[
            0x00, 0x0f, 0x19, 0x2f, b'0', b'5', b'9', 0x3a, 0x40, b'A', b'F', b'G', 0x5f, 0x60,
            b'a', b'f', b'g', 0x7f, 0x80, 0xb5, 0xf0, 0xff,
        ];

        // Two positions at a time out of every pair of edge bytes, over a word
        // which is otherwise digits, which covers every way two of the ranges
        // can meet without the run taking all day.
        for a in EDGES {
            for b in EDGES {
                for i in 0..8 {
                    for j in 0..8 {
                        let mut w = *b"0123abcd";
                        w[i] = *a;
                        w[j] = *b;

                        assert_eq!(dec8(w), reference(w, 10), "decimal {w:?}");
                        assert_eq!(hex8(w), reference(w, 16), "hexadecimal {w:?}");
                    }
                }
            }
        }
    }

    /// The two ends of the range, which are what the folding has to carry
    /// without losing a digit.
    #[test]
    fn extremes() {
        assert_eq!(dec8(*b"00000000"), Some(0));
        assert_eq!(dec8(*b"99999999"), Some(99_999_999));
        assert_eq!(dec8(*b"12345678"), Some(12_345_678));
        assert_eq!(dec8(*b"00000001"), Some(1));
        assert_eq!(dec8(*b"10000000"), Some(10_000_000));

        assert_eq!(hex8(*b"00000000"), Some(0));
        assert_eq!(hex8(*b"ffffffff"), Some(u32::MAX));
        assert_eq!(hex8(*b"FFFFFFFF"), Some(u32::MAX));
        assert_eq!(hex8(*b"0123abcd"), Some(0x0123_abcd));
        assert_eq!(hex8(*b"0123ABCD"), Some(0x0123_abcd));
        assert_eq!(hex8(*b"DeadBeef"), Some(0xdead_beef));
        assert_eq!(hex8(*b"00000001"), Some(1));
        assert_eq!(hex8(*b"10000000"), Some(0x1000_0000));
    }

    /// Reading a word out of the middle of a buffer.
    #[test]
    fn reading_a_word() {
        let buf = b"xx12345678yy";
        assert_eq!(word(buf, 2), *b"12345678");
        assert_eq!(dec8(word(buf, 2)), Some(12_345_678));
        assert_eq!(dec8(word(buf, 0)), None);
    }
}
