//!
//! Analog Inputs
//! 

use derive_builder::Builder;
use defmt::Format;
use crate::packing::{Pack, PackingError, Unpack};

#[derive(Clone, Copy, Debug, Format, PartialEq, Eq, Default, Builder)]
#[builder(build_fn(error(validation_error = false)))]
/// Analog Inputs from various sources
pub struct AnalogInputs {
    #[builder(default = "0")]
    /// The first analog input
    pub a0: u16,

    #[builder(default = "0")]
    /// The second analog input
    pub a1: u16,

    #[builder(default = "0")]
    /// The third analog input
    pub a2: u16,

    #[builder(default = "0")]
    /// The fourth analog input
    pub a3: u16,

    #[builder(default = "0")]
    /// The fifth analog input
    pub a4: u16,

    #[builder(default = "0")]
    /// THe sixth analog input
    pub a5: u16,
}

impl Pack for AnalogInputs {
    fn pack(self, buffer: &mut [u8]) -> Result<(), PackingError> {
        if buffer.len() < 12 {
            return Err(PackingError::InvalidBufferSize);
        }

        buffer[0..2].copy_from_slice(&self.a0.to_le_bytes());
        buffer[2..4].copy_from_slice(&self.a1.to_le_bytes());
        buffer[4..6].copy_from_slice(&self.a2.to_le_bytes());
        buffer[6..8].copy_from_slice(&self.a3.to_le_bytes());
        buffer[8..10].copy_from_slice(&self.a4.to_le_bytes());
        buffer[10..12].copy_from_slice(&self.a5.to_le_bytes());

        Ok(())
    }
}

impl Unpack for AnalogInputs {
    fn unpack(buffer: &[u8]) -> Result<Self, PackingError> where Self: Sized {
        if buffer.len() < 12 {
            return Err(PackingError::InvalidBufferSize);
        }

        Ok(Self {
            a0: u16::from_le_bytes(buffer[0..2].try_into().unwrap()),
            a1: u16::from_le_bytes(buffer[2..4].try_into().unwrap()),
            a2: u16::from_le_bytes(buffer[4..6].try_into().unwrap()),
            a3: u16::from_le_bytes(buffer[6..8].try_into().unwrap()),
            a4: u16::from_le_bytes(buffer[8..10].try_into().unwrap()),
            a5: u16::from_le_bytes(buffer[10..12].try_into().unwrap()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_analog_inputs() {
        let analog_inputs = AnalogInputs {
            a0: 0x1234,
            a1: 0x2345,
            a2: 0x3456,
            a3: 0x4567,
            a4: 0x5678,
            a5: 0x6789,
        };

        let mut buffer = [0u8; 12];
        analog_inputs.pack(&mut buffer).unwrap();
        assert_eq!(
            buffer,
            [0x34, 0x12, 0x45, 0x23, 0x56, 0x34, 0x67, 0x45, 0x78, 0x56, 0x89, 0x67]
        );
    }

    #[test]
    fn test_unpack_analog_inputs() {
        let buffer = [0x23, 0x12, 0x45, 0x34, 0x67, 0x56, 0x89, 0x78, 0x01, 0x90, 0x23, 0x12];
        let inputs = AnalogInputs::unpack(&buffer).unwrap();

        assert_eq!(
            inputs,
            AnalogInputs {
                a0: 0x1223,
                a1: 0x3445,
                a2: 0x5667,
                a3: 0x7889,
                a4: 0x9001,
                a5: 0x1223,
            }
        )
    }

    #[test]
    fn test_pack_unpack_analog_inputs() {
        let analog_inputs = AnalogInputs {
            a0: 0x1234,
            a1: 0x2345,
            a2: 0x3456,
            a3: 0x4567,
            a4: 0x5678,
            a5: 0x6789,
        };

        let mut buffer = [0u8; 12];
        analog_inputs.clone().pack(&mut buffer).unwrap();
        assert_eq!(
            analog_inputs,
            AnalogInputs::unpack(&buffer).unwrap(),
        );
    }
}