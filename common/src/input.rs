//!
//! Common abstractions and operations for the Input Modules
//!

use core::{fmt::Debug, ops::{BitOr, BitOrAssign}};
use derive_builder::Builder;
use defmt::Format;

use embedded_hal::i2c::{SevenBitAddress, I2c};
pub mod numpad;
use numpad::Numpad;

pub mod keypad;
use keypad::Keypad;

pub mod auxiliary;
use auxiliary::Auxiliary;

pub mod analog;
use analog::AnalogInputs;

pub mod other;
use other::{OtherInput, DecodeInstructions};

use crate::packing::{Pack, PackingError, Unpack};

/// Driver for programming modules to use to interface with the main input module
pub struct InputModuleDriver<I2C> {
    /// The address of the input module
    address: SevenBitAddress,
    /// The i2c peripheral
    i2c: I2C,
}

impl<I2C, I2CErr> InputModuleDriver<I2C> where
I2C: I2c<SevenBitAddress, Error=I2CErr>,
I2CErr: Debug + Format {
    /// Initialize a new Input Module Driver
    pub fn new(address: u8, i2c: I2C) -> Self {
        Self {
            address: address.into(),
            i2c,
        }
    }

    /// Set the i2c address for the main input module
    pub fn set_address(&mut self, new_address: u8) -> Result<(), I2CErr> {
        let buffer = [InputRequest::SetAddress as u8, new_address];
        self.i2c.write(self.address, &buffer)?;
        self.address = new_address;
        Ok(())
    }

    /// Get the full input information from the main input module
    pub fn get_input(&mut self) -> Result<Input, I2CErr> {
        let instruction = [InputRequest::FullInput as u8];
        let mut buffer = [0u8; 71];
        self.i2c.write_read(self.address, &instruction, &mut buffer)?;
        Ok(Input::unpack(&buffer).unwrap())
    }

    /// Get the numpad input information from the main input module
    pub fn get_numpad(&mut self) -> Result<Numpad, I2CErr> {
        let instruction = [InputRequest::Numpad as u8];
        let mut buffer = [0u8; 2];
        self.i2c.write_read(self.address, &instruction, &mut buffer)?;
        Ok(Numpad::unpack(&buffer).unwrap())
    }

    /// Get the keypad input information from the main input module
    pub fn get_keypad(&mut self) -> Result<Keypad, I2CErr> {
        let instruction = [InputRequest::Keypad as u8];
        let mut buffer = [0u8; 4];
        self.i2c.write_read(self.address, &instruction, &mut buffer)?;
        Ok(Keypad::unpack(&buffer).unwrap())
    }

    /// Get the auxiliary input information from the main input module
    pub fn get_auxiliary(&mut self) -> Result<Auxiliary, I2CErr> {
        let instruction = [InputRequest::Auxiliary as u8];
        let mut buffer = [0u8; 4];
        self.i2c.write_read(self.address, &instruction, &mut buffer)?;
        Ok(Auxiliary::unpack(&buffer).unwrap())
    }

    /// Get the analog input information from the main input module
    pub fn get_analog(&mut self) -> Result<AnalogInputs, I2CErr> {
        let instruction = [InputRequest::Analog as u8];
        let mut buffer = [0u8; 12];
        self.i2c.write_read(self.address, &instruction, &mut buffer)?;
        Ok(AnalogInputs::unpack(&buffer).unwrap())
    }

    /// Get the decode instructions for the first other input module
    pub fn get_decode_one(&mut self) -> Result<DecodeInstructions, I2CErr> {
        let instruction = [InputRequest::DecodeOne as u8];
        let mut buffer = [0u8; 248];
        self.i2c.write_read(self.address, &instruction, &mut buffer)?;
        Ok(DecodeInstructions::unpack(&buffer).unwrap())
    }

    /// Get the input data for the first other input module
    pub fn get_other_one(&mut self) -> Result<OtherInput, I2CErr> {
        let instruction = [InputRequest::OtherOne as u8];
        let mut buffer = [0u8; 24];
        self.i2c.write_read(self.address, &instruction, &mut buffer)?;
        Ok(buffer)
    }

    /// Get the decode instructions for the second other input module
    pub fn get_decode_two(&mut self) -> Result<DecodeInstructions, I2CErr> {
        let instruction = [InputRequest::DecodeTwo as u8];
        let mut buffer = [0u8; 248];
        self.i2c.write_read(self.address, &instruction, &mut buffer)?;
        Ok(DecodeInstructions::unpack(&buffer).unwrap())
    }

    /// Get the input data for the second other input module
    pub fn get_other_two(&mut self) -> Result<OtherInput, I2CErr> {
        let instruction = [InputRequest::OtherTwo as u8];
        let mut buffer = [0u8; 24];
        self.i2c.write_read(self.address, &instruction, &mut buffer)?;
        Ok(buffer)
    }
}

#[derive(Clone, Copy, Debug, Format, PartialEq, Eq)]
/// Request type for getting input from the I/O Module
pub enum InputRequest {
    /// Request all fields of the input
    FullInput = 0x00,
    /// Request the numpad inputs
    Numpad = 0x01,
    /// Request the keypad inputs
    Keypad = 0x02,
    /// Request the auxiliary inputs
    Auxiliary = 0x03,
    /// Request the analog inputs
    Analog = 0x04,
    /// Request the other input decode instructions for I/O Module 1
    DecodeOne = 0x05,
    /// Request the other inputs from I/O Module 1
    OtherOne = 0x06,
    /// Request the other input decode instructions for I/O Module 2
    DecodeTwo = 0x07,
    /// Request the other inputs from I/O Module 2
    OtherTwo = 0x08,
    /// Set the I2C Address of the main input module
    SetAddress = 0x09,
}

impl From<u8> for InputRequest {
    fn from(value: u8) -> Self {
        match value {
            0 => InputRequest::FullInput,
            1 => InputRequest::Numpad,
            2 => InputRequest::Keypad,
            3 => InputRequest::Auxiliary,
            4 => InputRequest::Analog,
            5 => InputRequest::DecodeOne,
            6 => InputRequest::OtherOne,
            7 => InputRequest::DecodeTwo,
            8 => InputRequest::OtherTwo,
            _ => InputRequest::SetAddress,
        }
    }
}

#[derive(Clone, Copy, Debug, Format, PartialEq, Eq, Default, Builder)]
#[builder(build_fn(error(validation_error = false)))]
/// A struct containing the input from the input modules
pub struct Input {
    #[builder(default = "Numpad::default()")]
    /// Numpad input
    pub numpad: Numpad,

    #[builder(default = "Keypad::default()")]
    /// Keypad input
    pub keypad: Keypad,

    #[builder(default = "Auxiliary::default()")]
    /// Auxiliary input
    pub auxiliary: Auxiliary,

    #[builder(default = "AnalogInputs::default()")]
    /// Analog input
    pub analog: AnalogInputs,

    #[builder(default = "[0u8; 24]")]
    /// Other Input 1
    pub other_input_one: OtherInput,

    #[builder(default = "[0u8; 24]")]
    /// Other Input 2
    pub other_input_two: OtherInput,
}

impl Pack for Input {
    fn pack(self, buffer: &mut [u8]) -> Result<(), PackingError> {
        if buffer.len() < 71 {
            return Err(PackingError::InvalidBufferSize);
        }

        self.numpad.pack(&mut buffer[0..2])?;
        self.keypad.pack(&mut buffer[2..6])?;
        self.auxiliary.pack(&mut buffer[6..10])?;
        self.analog.pack(&mut buffer[10..22])?;
        buffer[22..46].copy_from_slice(&self.other_input_one);
        buffer[46..70].copy_from_slice(&self.other_input_two);
        Ok(())
    }
}

impl Unpack for Input {
    fn unpack(buffer: &[u8]) -> Result<Self, PackingError> where Self: Sized {
        if buffer.len() < 71 {
            return Err(PackingError::InvalidBufferSize);
        }

        Ok(Self {
            numpad: Numpad::unpack(&buffer[0..2])?,
            keypad: Keypad::unpack(&buffer[2..6])?,
            auxiliary: Auxiliary::unpack(&buffer[6..10])?,
            analog: AnalogInputs::unpack(&buffer[10..22])?,
            other_input_one: buffer[22..46].try_into().unwrap(),
            other_input_two: buffer[46..70].try_into().unwrap(),
        })
    }
}

impl BitOr for Input {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            numpad: self.numpad | rhs.numpad,
            keypad: self.keypad | rhs.keypad,
            auxiliary: self.auxiliary | rhs.auxiliary,
            analog: self.analog,
            other_input_one: self.other_input_one,
            other_input_two: self.other_input_two,
        }
    }
}

impl BitOrAssign for Input {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use keypad::KeypadBuilder;
    use auxiliary::AuxiliaryBuilder;

    #[test]
    fn test_pack_unpack_inputs() {
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
        let keypad = KeypadBuilder::default()
            .a(true)
            .c(true)
            .e(true)
            .g(true)
            .i(true)
            .k(true)
            .m(true)
            .o(true)
            .q(true)
            .s(true)
            .u(true)
            .w(true)
            .y(true)
            .build()
            .unwrap();
        let auxiliary = AuxiliaryBuilder::default()
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
        let analog_inputs = AnalogInputs {
            a0: 0x1234,
            a1: 0x2345,
            a2: 0x3456,
            a3: 0x4567,
            a4: 0x5678,
            a5: 0x6789,
        };

        let inputs = Input {
            numpad,
            keypad,
            auxiliary,
            analog: analog_inputs,
            other_input_one: [0u8; 24],
            other_input_two: [255u8; 24],
        };

        let mut buffer = [0u8; 71];
        inputs.clone().pack(&mut buffer).unwrap();

        let decoded_inputs = Input::unpack(&buffer).unwrap();

        assert_eq!(inputs, decoded_inputs)
    }
}
