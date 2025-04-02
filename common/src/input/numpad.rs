//!
//! Numpad Inputs
//! 

use core::ops::{BitOr, BitOrAssign};

use derive_builder::Builder;
use defmt::Format;
use crate::packing::{Pack, PackingError, Unpack};

#[derive(Clone, Copy, Debug, Format, PartialEq, Eq, Default, Builder)]
#[builder(build_fn(error(validation_error = false)))]
/// Numpad directives (i.e. numbers 0-9)
pub struct Numpad {
    #[builder(default = "false")]
    /// The zero button
    pub zero: bool,

    #[builder(default = "false")]
    /// The one button
    pub one: bool,

    #[builder(default = "false")]
    /// The two button
    pub two: bool,

    #[builder(default = "false")]
    /// The three button
    pub three: bool,

    #[builder(default = "false")]
    /// The four button
    pub four: bool,

    #[builder(default = "false")]
    /// The five button
    pub five: bool,

    #[builder(default = "false")]
    /// The six button
    pub six: bool,

    #[builder(default = "false")]
    /// The seven button
    pub seven: bool,

    #[builder(default = "false")]
    /// The eight button
    pub eight: bool,

    #[builder(default = "false")]
    /// The nine button
    pub nine: bool,
}

impl Pack for Numpad {
    fn pack(self, buffer: &mut [u8]) -> Result<(), PackingError> {
        if buffer.len() < 2 {
            return Err(PackingError::InvalidBufferSize);
        }

        buffer[0] = ((self.zero as u8) << 7)
            | ((self.one as u8) << 6)
            | ((self.two as u8) << 5)
            | ((self.three as u8) << 4)
            | ((self.four as u8) << 3)
            | ((self.five as u8) << 2)
            | ((self.six as u8) << 1)
            | (self.seven as u8);
        buffer[1] = ((self.eight as u8) << 7) | ((self.nine as u8) << 6);
        Ok(())
    }
}

impl Unpack for Numpad {
    fn unpack(buffer: &[u8]) -> Result<Self, PackingError>
    where
        Self: Sized,
    {
        if buffer.len() < 2 {
            return Err(PackingError::InvalidBufferSize);
        }

        Ok(Self {
            zero: buffer[0] & (1 << 7) != 0,
            one: buffer[0] & (1 << 6) != 0,
            two: buffer[0] & (1 << 5) != 0,
            three: buffer[0] & (1 << 4) != 0,
            four: buffer[0] & (1 << 3) != 0,
            five: buffer[0] & (1 << 2) != 0,
            six: buffer[0] & (1 << 1) != 0,
            seven: buffer[0] & 1 != 0,
            eight: buffer[1] & (1 << 7) != 0,
            nine: buffer[1] & (1 << 6) != 0,
        })
    }
}

impl BitOr for Numpad {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            zero: self.zero || rhs.zero,
            one: self.one || rhs.one,
            two: self.two || rhs.two,
            three: self.three || rhs.three,
            four: self.four || rhs.four,
            five: self.five || rhs.five,
            six: self.six || rhs.six,
            seven: self.seven || rhs.seven,
            eight: self.eight || rhs.eight,
            nine: self.nine || rhs.nine,
        }
    }
}

impl BitOrAssign for Numpad {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_numpad() {
        let numpad = Numpad {
            zero: false,
            one: true,
            two: false,
            three: true,
            four: false,
            five: true,
            six: false,
            seven: true,
            eight: false,
            nine: true,
        };

        let mut buffer = [0u8; 2];
        numpad.pack(&mut buffer).unwrap();

        assert_eq!(buffer, [0b0101_0101, 0b0100_0000]);
    }

    #[test]
    fn test_unpack_numpad() {
        let buffer = [0b1010_1010, 0b1000_0000];
        let numpad = Numpad::unpack(&buffer).unwrap();

        assert_eq!(numpad, Numpad {
            zero: true,
            one: false,
            two: true,
            three: false,
            four: true,
            five: false,
            six: true,
            seven: false,
            eight: true,
            nine: false
        });
    }

    #[test]
    fn test_pack_unpack_numpad() {
        let numpad = Numpad {
            zero: false,
            one: false,
            two: false,
            three: false,
            four: true,
            five: true,
            six: true,
            seven: true,
            eight: false,
            nine: false,
        };

        let mut buffer = [0u8; 2];
        numpad.clone().pack(&mut buffer).unwrap();
        assert_eq!(numpad, Numpad::unpack(&buffer).unwrap());
    }

    #[test]
    fn test_bitor_numpad() {
        let numpad1 = NumpadBuilder::create_empty()
            .zero(true)
            .two(true)
            .three(true)
            .eight(true)
            .nine(true)
            .build()
            .unwrap();
        let numpad2 = NumpadBuilder::create_empty()
            .zero(true)
            .four(true)
            .five(true)
            .nine(true)
            .build()
            .unwrap();
        let numpad = numpad1 | numpad2;
        assert!(numpad.zero);
        assert!(numpad.two);
        assert!(numpad.three);
        assert!(numpad.four);
        assert!(numpad.five);
        assert!(numpad.eight);
        assert!(numpad.nine);
    }
}