//!
//! Auxiliary Inputs
//! 

use core::ops::{BitOr, BitOrAssign};

use derive_builder::Builder;
use defmt::Format;
use crate::packing::{Pack, PackingError, Unpack};

#[derive(Clone, Copy, Debug, Format, PartialEq, Eq, Default, Builder)]
#[builder(build_fn(error(validation_error = false)))]
/// Auxiliary Characters
pub struct Auxiliary {
    #[builder(default = "false")]
    /// `!`
    pub exclamation: bool,

    #[builder(default = "false")]
    /// `@`
    pub at: bool,

    #[builder(default = "false")]
    /// `#`
    pub hash: bool,

    #[builder(default = "false")]
    /// `$`
    pub dollar: bool,

    #[builder(default = "false")]
    /// `%`
    pub percent: bool,

    #[builder(default = "false")]
    /// `^`
    pub caret: bool,

    #[builder(default = "false")]
    /// `&`
    pub and: bool,

    #[builder(default = "false")]
    /// `*`
    pub star: bool,

    #[builder(default = "false")]
    /// `(`
    pub left_paren: bool,

    #[builder(default = "false")]
    /// `)`
    pub right_paren: bool,

    #[builder(default = "false")]
    /// `-`
    pub minus: bool,

    #[builder(default = "false")]
    /// `_`
    pub underscore: bool,

    #[builder(default = "false")]
    /// `+'
    pub plus: bool,

    #[builder(default = "false")]
    /// `=`
    pub equal: bool,

    #[builder(default = "false")]
    /// '`'
    pub backtick: bool,

    #[builder(default = "false")]
    /// `~`
    pub tilde: bool,

    #[builder(default = "false")]
    /// `[`
    pub left_square: bool,

    #[builder(default = "false")]
    /// `]`
    pub right_square: bool,

    #[builder(default = "false")]
    /// `{`
    pub left_curly: bool,

    #[builder(default = "false")]
    /// `}`
    pub right_curly: bool,

    #[builder(default = "false")]
    /// `\`
    pub backslash: bool,

    #[builder(default = "false")]
    /// `|`
    pub pipe: bool,

    #[builder(default = "false")]
    /// `;`
    pub semicolon: bool,

    #[builder(default = "false")]
    /// `:`
    pub colon: bool,

    #[builder(default = "false")]
    /// `'`
    pub single_quote: bool,

    #[builder(default = "false")]
    /// `"`
    pub double_quote: bool,

    #[builder(default = "false")]
    /// `,`
    pub comma: bool,

    #[builder(default = "false")]
    /// `.`
    pub period: bool,

    #[builder(default = "false")]
    /// `<`
    pub less_than: bool,

    #[builder(default = "false")]
    /// `>`
    pub greater_than: bool,

    #[builder(default = "false")]
    /// `/`
    pub forwardslash: bool,

    #[builder(default = "false")]
    /// `?`
    pub question: bool,
}

impl Pack for Auxiliary {
    fn pack(self, buffer: &mut [u8]) -> Result<(), PackingError> {
        if buffer.len() < 4 {
            return Err(PackingError::InvalidBufferSize);
        }

        buffer[0] = ((self.exclamation as u8) << 7) |
            ((self.at as u8) << 6) |
            ((self.hash as u8) << 5) |
            ((self.dollar as u8) << 4) |
            ((self.percent as u8) << 3) |
            ((self.caret as u8) << 2) |
            ((self.and as u8) << 1) |
            self.star as u8;
        buffer[1] = ((self.left_paren as u8) << 7) |
            ((self.right_paren as u8) << 6) |
            ((self.minus as u8) << 5) |
            ((self.underscore as u8) << 4) |
            ((self.plus as u8) << 3) |
            ((self.equal as u8) << 2) |
            ((self.backtick as u8) << 1) |
            self.tilde as u8;
        buffer[2] = ((self.left_square as u8) << 7) |
            ((self.right_square as u8) << 6) |
            ((self.left_curly as u8) << 5) |
            ((self.right_curly as u8) << 4) |
            ((self.backslash as u8) << 3) |
            ((self.pipe as u8) << 2) |
            ((self.semicolon as u8) << 1) |
            self.colon as u8;
        buffer[3] = ((self.single_quote as u8) << 7) |
            ((self.double_quote as u8) << 6) |
            ((self.comma as u8) << 5) |
            ((self.period as u8) << 4) |
            ((self.less_than as u8) << 3) |
            ((self.greater_than as u8) << 2) |
            ((self.forwardslash as u8) << 1) |
            self.question as u8;

        Ok(())
    }
}

impl Unpack for Auxiliary {
    fn unpack(buffer: &[u8]) -> Result<Self, PackingError> where Self: Sized {
        if buffer.len() < 4 {
            return Err(PackingError::InvalidBufferSize);
        }

        Ok(Auxiliary {
            exclamation: buffer[0] & (1 << 7) != 0,
            at: buffer[0] & (1 << 6) != 0,
            hash: buffer[0] & (1 << 5) != 0,
            dollar: buffer[0] & (1 << 4) != 0,
            percent: buffer[0] & (1 << 3) != 0,
            caret: buffer[0] & (1 << 2) != 0,
            and: buffer[0] & (1 << 1) != 0,
            star: buffer[0] & 1 != 0,
            left_paren: buffer[1] & (1 << 7) != 0,
            right_paren: buffer[1] & (1 << 6) != 0,
            minus: buffer[1] & (1 << 5) != 0,
            underscore: buffer[1] & (1 << 4) != 0,
            plus: buffer[1] & (1 << 3) != 0,
            equal: buffer[1] & (1 << 2) != 0,
            backtick: buffer[1] & (1 << 1) != 0,
            tilde: buffer[1] & 1 != 0,
            left_square: buffer[2] & (1 << 7) != 0,
            right_square: buffer[2] & (1 << 6) != 0,
            left_curly: buffer[2] & (1 << 5) != 0,
            right_curly: buffer[2] & (1 << 4) != 0,
            backslash: buffer[2] & (1 << 3) != 0,
            pipe: buffer[2] & (1 << 2) != 0,
            semicolon: buffer[2] & (1 << 1) != 0,
            colon: buffer[2] & 1 != 0,
            single_quote: buffer[3] & (1 << 7) != 0,
            double_quote: buffer[3] & (1 << 6) != 0,
            comma: buffer[3] & (1 << 5) != 0,
            period: buffer[3] & (1 << 4) != 0,
            less_than: buffer[3] & (1 << 3) != 0,
            greater_than: buffer[3] & (1 << 2) != 0,
            forwardslash: buffer[3] & (1 << 1) != 0,
            question: buffer[3] & 1 != 0,
        })
    }
}

impl BitOr for Auxiliary {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            exclamation: self.exclamation || rhs.exclamation,
            at: self.at || rhs.at,
            hash: self.hash || rhs.hash,
            dollar: self.dollar || rhs.dollar,
            percent: self.percent || rhs.percent,
            caret: self.caret || rhs.caret,
            and: self.and || rhs.and,
            star: self.star || rhs.star,
            left_curly: self.left_curly || rhs.left_curly,
            right_curly: self.right_curly || rhs.right_curly,
            right_paren: self.right_paren || rhs.right_paren,
            left_paren: self.left_paren || rhs.left_paren,
            minus: self.minus || rhs.minus,
            underscore: self.underscore || rhs.underscore,
            plus: self.plus || rhs.plus,
            equal: self.equal || rhs.equal,
            backslash: self.backslash || rhs.backslash,
            backtick: self.backtick || rhs.backtick,
            tilde: self.tilde || rhs.tilde,
            left_square: self.left_square || rhs.left_square,
            right_square: self.right_square || rhs.right_square,
            pipe: self.pipe || rhs.pipe,
            semicolon: self.semicolon || rhs.semicolon,
            colon: self.colon || rhs.colon,
            single_quote: self.single_quote || rhs.single_quote,
            double_quote: self.double_quote || rhs.double_quote,
            comma: self.comma || rhs.comma,
            period: self.period || rhs.period,
            less_than: self.less_than || rhs.less_than,
            greater_than: self.greater_than || rhs.greater_than,
            forwardslash: self.forwardslash || rhs.forwardslash,
            question: self.question || rhs.question,
        }
    }
}

impl BitOrAssign for Auxiliary {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_auxiliary() {
        let auxiliary = AuxiliaryBuilder::create_empty()
            .exclamation(true)
            .hash(true)
            .percent(true)
            .and(true)
            .left_paren(true)
            .minus(true)
            .plus(true)
            .backtick(true)
            .left_square(true)
            .left_curly(true)
            .backslash(true)
            .semicolon(true)
            .single_quote(true)
            .comma(true)
            .less_than(true)
            .forwardslash(true)
            .build()
            .unwrap();

        let mut buffer = [0u8; 4];
        auxiliary.pack(&mut buffer).unwrap();
        assert_eq!(
            buffer,
            [0b1010_1010, 0b1010_1010, 0b1010_1010, 0b1010_1010],
        );
    }

    #[test]
    fn test_unpack_auxiliary() {
        let buffer = [0b0101_0101, 0b0101_0101, 0b0101_0101, 0b0101_0101];

        let auxiliary = Auxiliary::unpack(&buffer).unwrap();
        assert_eq!(
            auxiliary,
            AuxiliaryBuilder::create_empty()
                .at(true)
                .dollar(true)
                .caret(true)
                .star(true)
                .right_paren(true)
                .underscore(true)
                .equal(true)
                .tilde(true)
                .right_square(true)
                .right_curly(true)
                .pipe(true)
                .colon(true)
                .double_quote(true)
                .period(true)
                .greater_than(true)
                .question(true)
                .build()
                .unwrap(),
        );
    }

    #[test]
    fn test_pack_unpack_auxiliary() {
        let auxiliary = AuxiliaryBuilder::create_empty()
            .exclamation(true)
            .hash(true)
            .percent(true)
            .and(true)
            .left_paren(true)
            .minus(true)
            .plus(true)
            .backtick(true)
            .left_square(true)
            .left_curly(true)
            .backslash(true)
            .semicolon(true)
            .single_quote(true)
            .comma(true)
            .less_than(true)
            .forwardslash(true)
            .build()
            .unwrap();

        let mut buffer = [0u8; 4];
        auxiliary.clone().pack(&mut buffer).unwrap();
        assert_eq!(
            auxiliary,
            Auxiliary::unpack(&buffer).unwrap(),
        );
    }

    #[test]
    fn test_bitor_auxiliary() {
        let auxiliary1 = AuxiliaryBuilder::create_empty()
            .exclamation(true)
            .plus(true)
            .forwardslash(true)
            .build()
            .unwrap();
        let auxiliary2 = AuxiliaryBuilder::create_empty()
            .exclamation(true)
            .forwardslash(true)
            .comma(true)
            .build()
            .unwrap();
        let auxiliary = auxiliary1 | auxiliary2;
        assert!(auxiliary.exclamation);
        assert!(auxiliary.plus);
        assert!(auxiliary.forwardslash);
        assert!(auxiliary.forwardslash);
        assert!(auxiliary.comma);
    }
}