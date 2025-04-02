//!
//! Other Inputs
//! 

use defmt::Format;
use crate::packing::{Pack, PackingError, Unpack};

/// The data storage for other inputs
/// 
/// Data Encoded into the Other Input Field Should use Little Endian Encodings
pub type OtherInput = [u8; 24];

#[derive(Clone, Copy, Debug, Format, PartialEq, Eq)]
/// The size of data used in the decode instruction information
pub enum DataSize {
    /// One Byte (0b1)
    One = 1,
    /// Two Bytes (0b01)
    Two = 2,
    /// Four Bytes (0b001)
    Four = 4,
    /// Eight bytes (0b0001)
    Eight = 8,
}

#[derive(Clone, Copy, Debug, Format, PartialEq, Eq)]
/// The type of data used in the decode instruction information
pub enum DataType {
    /// Unisigned Integer (0b1)
    Unsigned,
    /// Signed Integer (0b01)
    Signed,
    /// Floating Point (0b001)
    Floating,
}

#[derive(Clone, Copy, Debug, Format, PartialEq, Eq)]
/// For other input, all buffers must be 24 bytes in length.  Within this buffer, the
/// data can be decoded in any way.  Specifically, in this case, the data will be decoded 
/// with respect to these instructions
pub struct DecodeInstructions {
    /// The unique id of the input module
    pub module_id: u16,
    /// The size of each piece of data
    pub data_sizes: [DataSize; 24],
    /// The data type of each piece of data
    pub data_types: [DataType; 24],
    /// The names of each field (ascii)
    pub fields: [[u8; 10]; 24],
}

impl Default for DecodeInstructions {
    fn default() -> Self {
        Self {
            module_id: 0,
            data_sizes: [DataSize::One; 24],
            data_types: [DataType::Unsigned; 24],
            fields: [[0u8; 10]; 24],
        }
    }
}

impl Pack for [DataSize; 24] {
    fn pack(self, buffer: &mut [u8]) -> Result<(), PackingError> {
        if buffer.len() < 3 {
            return Err(PackingError::InvalidBufferSize);
        }

        let mut value = 0u32;
        let mut bit_index = 0;
        let mut cumulative_length = 0;
        for data_size in self {
            if cumulative_length >= 24 {
                break;
            }

            match data_size {
                DataSize::One => {
                    value |= 1 << bit_index;
                    bit_index += 1;
                    cumulative_length += 1;
                },
                DataSize::Two => {
                    value |= 1 << (bit_index + 1);
                    bit_index += 2;
                    cumulative_length += 2;
                },
                DataSize::Four => {
                    value |= 1 << (bit_index + 2);
                    bit_index += 3;
                    cumulative_length += 4;
                },
                DataSize::Eight => {
                    value |= 1 << (bit_index + 3);
                    bit_index += 4;
                    cumulative_length += 8;
                }
            }
        }

        let bytes = value.to_le_bytes();

        buffer[0..3].copy_from_slice(bytes[0..3].try_into().unwrap());

        Ok(())
    }
}

impl Unpack for [DataSize; 24] {
    fn unpack(buffer: &[u8]) -> Result<Self, PackingError> where Self: Sized {
        if buffer.len() < 3 {
            return Err(PackingError::InvalidBufferSize);
        }

        let data = [buffer[0], buffer[1], buffer[2], 0];
        let data = u32::from_le_bytes(data);

        let mut data_sizes = [DataSize::One; 24];
        let mut data_sizes_index = 0;
        let mut bit_index = 0;
        let mut cumulative_length = 0;
        while cumulative_length < 24 {
            if data & (0b1111 << bit_index) == 0b1000 << bit_index {
                data_sizes[data_sizes_index] = DataSize::Eight;
                bit_index += 4;
                cumulative_length += 8;
                data_sizes_index += 1;
            } else if data & (0b111 << bit_index) == 0b100 << bit_index {
                data_sizes[data_sizes_index] = DataSize::Four;
                bit_index += 3;
                cumulative_length += 4;
                data_sizes_index += 1;
            } else if data & (0b11 << bit_index) == 0b10 << bit_index {
                data_sizes[data_sizes_index] = DataSize::Two;
                bit_index += 2;
                cumulative_length += 2;
                data_sizes_index += 1;
            } else if data & (0b1 << bit_index) == 0b1 << bit_index {
                data_sizes[data_sizes_index] = DataSize::One;
                bit_index += 1;
                cumulative_length += 1;
                data_sizes_index += 1;
            }
        }

        Ok(data_sizes)
    }
}

impl Pack for [DataType; 24] {
    fn pack(self, buffer: &mut [u8]) -> Result<(), PackingError> {
        if buffer.len() < 3 {
            return Err(PackingError::InvalidBufferSize);
        }

        let mut value = 0u32;
        let mut bit_index = 0;
        for data_type in self {
            if bit_index >= 24 {
                break;
            }

            match data_type {
                DataType::Unsigned => {
                    value |= 1 << bit_index;
                    bit_index += 1;
                },
                DataType::Signed => {
                    value |= 1 << (bit_index + 1);
                    bit_index += 2;
                },
                DataType::Floating => {
                    value |= 1 << (bit_index + 2);
                    bit_index += 3;
                }
            }
        }

        let bytes = value.to_le_bytes();

        buffer[0..3].copy_from_slice(bytes[0..3].try_into().unwrap());

        Ok(())
    }
}

impl Unpack for [DataType; 24] {
    fn unpack(buffer: &[u8]) -> Result<Self, PackingError> where Self: Sized {
        if buffer.len() < 3 {
            return Err(PackingError::InvalidBufferSize);
        }

        let data = [buffer[0], buffer[1], buffer[2], 0];
        let data = u32::from_le_bytes(data);

        let mut data_types = [DataType::Unsigned; 24];
        let mut data_types_index = 0;
        let mut bit_index = 0;
        while bit_index < 24 {
            if data & (0b111 << bit_index) == 0b100 << bit_index {
                data_types[data_types_index] = DataType::Floating;
                bit_index += 3;
                data_types_index += 1;
            } else if data & (0b11 << bit_index) == 0b10 << bit_index {
                data_types[data_types_index] = DataType::Signed;
                bit_index += 2;
                data_types_index += 1;
            } else if data & (0b1 << bit_index) == 0b1 << bit_index {
                data_types[data_types_index] = DataType::Unsigned;
                bit_index += 1;
                data_types_index += 1;
            }
        }

        Ok(data_types)
    }
}

impl Pack for DecodeInstructions {
    fn pack(self, buffer: &mut [u8]) -> Result<(), PackingError> {
        if buffer.len() < 248 {
            return Err(PackingError::InvalidBufferSize);
        }

        buffer[0..2].copy_from_slice(&self.module_id.to_le_bytes());

        self.data_sizes.pack(&mut buffer[2..5])?;
        self.data_types.pack(&mut buffer[5..8])?;

        for (i, field) in self.fields.iter().enumerate() {
            buffer[(8+(i*10))..(8+((i+1)*10))].copy_from_slice(field);
        }

        Ok(())
    }
}

impl Unpack for DecodeInstructions {
    fn unpack(buffer: &[u8]) -> Result<Self, PackingError> where Self: Sized {
        if buffer.len() < 8 {
            return Err(PackingError::InvalidBufferSize);
        }

        let module_id = u16::from_le_bytes(buffer[0..2].try_into().unwrap());

        let data_sizes = <[DataSize; 24]>::unpack(&buffer[2..5])?;
        let data_types = <[DataType; 24]>::unpack(&buffer[5..8])?;

        let mut fields = [[0u8; 10]; 24];
        for i in 0..24 {
            fields[i] = buffer[(8+(i*10))..(8+((i+1)*10))].try_into().unwrap();
        }

        Ok(Self {
            module_id,
            data_sizes,
            data_types,
            fields,
        })
    }
}

#[derive(Clone, Copy, Debug, Format, PartialEq)]
/// Decoded Value from Other Input Using the Decode Instructions
pub enum DecodedInput<'a> {
    /// A u8
    U8{ value: u8, name: &'a[u8; 10]},
    /// A u16
    U16{ value: u16, name: &'a[u8; 10]},
    /// A u32
    U32{ value: u32, name: &'a[u8; 10]},
    /// A u64
    U64{ value: u64, name: &'a[u8; 10]},
    /// An i8
    I8{ value: i8, name: &'a[u8; 10]},
    /// An i16
    I16{ value: i16, name: &'a[u8; 10]},
    /// An i32
    I32{ value: i32, name: &'a[u8; 10]},
    /// An i64
    I64{ value: i64, name: &'a[u8; 10]},
    /// An f32
    F32{ value: f32, name: &'a[u8; 10]},
    /// An f64
    F64{ value: f64, name: &'a[u8; 10]},
}

#[derive(Clone, Copy, Debug, Format, PartialEq, Eq)]
/// Error from attempting to inoppropriately decode information from Other Input
pub enum DecodeError {
    /// The requested data index is out of bounds
    OutOfBounds,
    /// The requested data type is unknown (this is likely to occur for 8 or 16 bit floats)
    UnknownDataType,
}

pub trait Decode<'a> {
    fn decode(&self, idx: usize, decode_instructions: &'a DecodeInstructions) -> Result<DecodedInput<'a>, DecodeError>;
}

impl<'a> Decode<'a> for OtherInput {
    fn decode(&self, idx: usize, decode_instructions: &'a DecodeInstructions) -> Result<DecodedInput<'a>, DecodeError> {
        let mut cumulative_counter = 0;
        for i in 0..idx {
            cumulative_counter += decode_instructions.data_sizes[i] as usize;
        }

        if cumulative_counter + decode_instructions.data_sizes[idx] as usize > 24 {
            return Err(DecodeError::OutOfBounds);
        }

        match (decode_instructions.data_sizes[idx], decode_instructions.data_types[idx]) {
            (DataSize::One, DataType::Unsigned) => Ok(DecodedInput::U8 {
                value: self[cumulative_counter],
                name: &decode_instructions.fields[idx],
            }),
            (DataSize::Two, DataType::Unsigned) => Ok(DecodedInput::U16 {
                value: u16::from_le_bytes(self[cumulative_counter..(cumulative_counter+2)].try_into().unwrap()),
                name: &decode_instructions.fields[idx],
            }),
            (DataSize::Four, DataType::Unsigned) => Ok(DecodedInput::U32 {
                value: u32::from_le_bytes(self[cumulative_counter..(cumulative_counter+4)].try_into().unwrap()),
                name: &decode_instructions.fields[idx],
            }),
            (DataSize::Eight, DataType::Unsigned) => Ok(DecodedInput::U64 {
                value: u64::from_le_bytes(self[cumulative_counter..(cumulative_counter+8)].try_into().unwrap()),
                name: &decode_instructions.fields[idx],
            }),
            (DataSize::One, DataType::Signed) => Ok(DecodedInput::I8 {
                value: i8::from_le_bytes([self[cumulative_counter]]),
                name: &decode_instructions.fields[idx],
            }),
            (DataSize::Two, DataType::Signed) => Ok(DecodedInput::I16 {
                value: i16::from_le_bytes(self[cumulative_counter..(cumulative_counter+2)].try_into().unwrap()),
                name: &decode_instructions.fields[idx],
            }),
            (DataSize::Four, DataType::Signed) => Ok(DecodedInput::I32 {
                value: i32::from_le_bytes(self[cumulative_counter..(cumulative_counter+4)].try_into().unwrap()),
                name: &decode_instructions.fields[idx],
            }),
            (DataSize::Eight, DataType::Signed) => Ok(DecodedInput::I64 {
                value: i64::from_le_bytes(self[cumulative_counter..(cumulative_counter+8)].try_into().unwrap()),
                name: &decode_instructions.fields[idx],
            }),
            (DataSize::Four, DataType::Floating) => Ok(DecodedInput::F32 {
                value: f32::from_le_bytes(self[cumulative_counter..(cumulative_counter+4)].try_into().unwrap()),
                name: &decode_instructions.fields[idx],
            }),
            (DataSize::Eight, DataType::Floating) => Ok(DecodedInput::F64 {
                value: f64::from_le_bytes(self[cumulative_counter..(cumulative_counter+8)].try_into().unwrap()),
                name: &decode_instructions.fields[idx],
            }),
            _ => Err(DecodeError::UnknownDataType),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_data_sizes() {
        let mut buffer = [0u8; 3];

        let mut data_sizes = [DataSize::One; 24];
        data_sizes[1] = DataSize::Two;
        data_sizes[2] = DataSize::Four;
        data_sizes[3] = DataSize::Eight;

        data_sizes.pack(&mut buffer).unwrap();

        let expected_buffer = [0b00100101, 0b11111110, 0b00000111];

        assert_eq!(buffer, expected_buffer);
    }

    #[test]
    fn test_unpack_data_sizes() {
        let buffer = [0b00100101, 0b11111110, 0b00000111];

        let data_sizes = <[DataSize; 24]>::unpack(&buffer).unwrap();
        
        let mut expected_data_sizes = [DataSize::One; 24];
        expected_data_sizes[1] = DataSize::Two;
        expected_data_sizes[2] = DataSize::Four;
        expected_data_sizes[3] = DataSize::Eight;

        assert_eq!(data_sizes, expected_data_sizes);
    }

    #[test]
    fn test_pack_data_types() {
        let mut buffer = [0u8; 3];

        let mut data_types = [DataType::Unsigned; 24];
        data_types[1] = DataType::Signed;
        data_types[2] = DataType::Floating;

        data_types.pack(&mut buffer).unwrap();

        let expected_buffer = [0b1110_0101, 0b1111_1111, 0b1111_1111];

        assert_eq!(buffer, expected_buffer);
    }

    #[test]
    fn test_unpack_data_types() {
        let buffer = [0b1110_0101, 0b1111_1111, 0b1111_1111];

        let data_types = <[DataType; 24]>::unpack(&buffer).unwrap();

        let mut expected_data_types = [DataType::Unsigned; 24];
        expected_data_types[1] = DataType::Signed;
        expected_data_types[2] = DataType::Floating;

        assert_eq!(data_types, expected_data_types);
    }

    #[test]
    fn test_pack_other_input_decode_instructions() {
        let mut data_sizes = [DataSize::One; 24];
        data_sizes[1] = DataSize::Two;
        data_sizes[2] = DataSize::Four;
        data_sizes[3] = DataSize::Eight;

        let mut data_types = [DataType::Unsigned; 24];
        data_types[1] = DataType::Signed;
        data_types[2] = DataType::Floating;

        let mut fields = [[0u8; 10]; 24];
        fields[0][0..5].copy_from_slice(b"test0");
        fields[1][0..5].copy_from_slice(b"test1");
        fields[2][0..5].copy_from_slice(b"test2");
        fields[3][0..5].copy_from_slice(b"test3");
        fields[4][0..5].copy_from_slice(b"test4");
        fields[5][0..5].copy_from_slice(b"test5");
        fields[6][0..5].copy_from_slice(b"test6");
        fields[7][0..5].copy_from_slice(b"test7");
        fields[8][0..5].copy_from_slice(b"test8");
        fields[9][0..5].copy_from_slice(b"test9");

        let decode_instruction = DecodeInstructions {
            module_id: 0x1212,
            data_sizes,
            data_types,
            fields
        };

        let mut buffer = [0u8; 248];
        decode_instruction.pack(&mut buffer).unwrap();

        let mut expected_buffer = [0u8; 248];
        // Module Id
        expected_buffer[0] = 0x12;
        expected_buffer[1] = 0x12;
        // Data Sizes
        expected_buffer[2] = 0b00100101;
        expected_buffer[3] = 0b11111110;
        expected_buffer[4] = 0b00000111;
        // Data Types
        expected_buffer[5] = 0b1110_0101;
        expected_buffer[6] = 0b1111_1111;
        expected_buffer[7] = 0b1111_1111;
        // Fields
        expected_buffer[8..13].copy_from_slice(b"test0");
        expected_buffer[18..23].copy_from_slice(b"test1");
        expected_buffer[28..33].copy_from_slice(b"test2");
        expected_buffer[38..43].copy_from_slice(b"test3");
        expected_buffer[48..53].copy_from_slice(b"test4");
        expected_buffer[58..63].copy_from_slice(b"test5");
        expected_buffer[68..73].copy_from_slice(b"test6");
        expected_buffer[78..83].copy_from_slice(b"test7");
        expected_buffer[88..93].copy_from_slice(b"test8");
        expected_buffer[98..103].copy_from_slice(b"test9");

        assert_eq!(expected_buffer, buffer);
    }

    #[test]
    fn test_unpack_other_input_decode_instructions() {
        let mut buffer = [0u8; 248];
        // Module Id
        buffer[0] = 0x12;
        buffer[1] = 0x12;
        // Data Sizes
        buffer[2] = 0b00100101;
        buffer[3] = 0b11111110;
        buffer[4] = 0b00000111;
        // Data Types
        buffer[5] = 0b1110_0101;
        buffer[6] = 0b1111_1111;
        buffer[7] = 0b1111_1111;
        // Fields
        buffer[8..13].copy_from_slice(b"test0");
        buffer[18..23].copy_from_slice(b"test1");
        buffer[28..33].copy_from_slice(b"test2");
        buffer[38..43].copy_from_slice(b"test3");
        buffer[48..53].copy_from_slice(b"test4");
        buffer[58..63].copy_from_slice(b"test5");
        buffer[68..73].copy_from_slice(b"test6");
        buffer[78..83].copy_from_slice(b"test7");
        buffer[88..93].copy_from_slice(b"test8");
        buffer[98..103].copy_from_slice(b"test9");

        let decode_instruction = DecodeInstructions::unpack(&buffer).unwrap();

        let mut data_sizes = [DataSize::One; 24];
        data_sizes[1] = DataSize::Two;
        data_sizes[2] = DataSize::Four;
        data_sizes[3] = DataSize::Eight;

        let mut data_types = [DataType::Unsigned; 24];
        data_types[1] = DataType::Signed;
        data_types[2] = DataType::Floating;

        let mut fields = [[0u8; 10]; 24];
        fields[0][0..5].copy_from_slice(b"test0");
        fields[1][0..5].copy_from_slice(b"test1");
        fields[2][0..5].copy_from_slice(b"test2");
        fields[3][0..5].copy_from_slice(b"test3");
        fields[4][0..5].copy_from_slice(b"test4");
        fields[5][0..5].copy_from_slice(b"test5");
        fields[6][0..5].copy_from_slice(b"test6");
        fields[7][0..5].copy_from_slice(b"test7");
        fields[8][0..5].copy_from_slice(b"test8");
        fields[9][0..5].copy_from_slice(b"test9");

        let expected_instruction = DecodeInstructions {
            module_id: 0x1212,
            data_sizes,
            data_types,
            fields
        };

        assert_eq!(expected_instruction, decode_instruction);
    }

    #[test]
    fn test_pack_unpack_other_input_decode_instructions() {
        let mut data_sizes = [DataSize::One; 24];
        data_sizes[1] = DataSize::Two;
        data_sizes[2] = DataSize::Four;
        data_sizes[3] = DataSize::Eight;

        let mut data_types = [DataType::Unsigned; 24];
        data_types[1] = DataType::Signed;
        data_types[2] = DataType::Floating;

        let mut fields = [[0u8; 10]; 24];
        fields[0][0..5].copy_from_slice(b"test0");
        fields[1][0..5].copy_from_slice(b"test1");
        fields[2][0..5].copy_from_slice(b"test2");
        fields[3][0..5].copy_from_slice(b"test3");
        fields[4][0..5].copy_from_slice(b"test4");
        fields[5][0..5].copy_from_slice(b"test5");
        fields[6][0..5].copy_from_slice(b"test6");
        fields[7][0..5].copy_from_slice(b"test7");
        fields[8][0..5].copy_from_slice(b"test8");
        fields[9][0..5].copy_from_slice(b"test9");

        let decode_instruction = DecodeInstructions {
            module_id: 0x1212,
            data_sizes,
            data_types,
            fields
        };

        let mut buffer = [0u8; 248];
        decode_instruction.clone().pack(&mut buffer).unwrap();

        let instruction = DecodeInstructions::unpack(&buffer).unwrap();
        assert_eq!(instruction, decode_instruction);
    }

    #[test]
    fn test_decode_u8() {
        let mut decode_instructions = DecodeInstructions::default();
        decode_instructions.fields[1] = *b"dinosaur__";

        let mut input = [0u8; 24];
        input[1] = 255;

        if let DecodedInput::U8 { value, name } = input.decode(1, &decode_instructions).unwrap() {
            assert_eq!(value, 255);
            assert_eq!(name, b"dinosaur__");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_decode_u16() {
        let mut decode_instructions = DecodeInstructions::default();
        decode_instructions.data_sizes[1] = DataSize::Two;
        decode_instructions.fields[1] = *b"dinosaur__";

        let mut input = [0u8; 24];
        input[1..3].copy_from_slice(&0x1234u16.to_le_bytes());

        if let DecodedInput::U16 { value, name } = input.decode(1, &decode_instructions).unwrap() {
            assert_eq!(value, 0x1234);
            assert_eq!(name, b"dinosaur__");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_decode_u32() {
        let mut decode_instructions = DecodeInstructions::default();
        decode_instructions.data_sizes[1] = DataSize::Four;
        decode_instructions.fields[1] = *b"dinosaur__";

        let mut input = [0u8; 24];
        input[1..5].copy_from_slice(&0x12345678u32.to_le_bytes());

        if let DecodedInput::U32 { value, name } = input.decode(1, &decode_instructions).unwrap() {
            assert_eq!(value, 0x12345678);
            assert_eq!(name, b"dinosaur__");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_decode_u64() {
        let mut decode_instructions = DecodeInstructions::default();
        decode_instructions.data_sizes[1] = DataSize::Eight;
        decode_instructions.fields[1] = *b"dinosaur__";

        let mut input = [0u8; 24];
        input[1..9].copy_from_slice(&0x123456789012u64.to_le_bytes());

        if let DecodedInput::U64 { value, name } = input.decode(1, &decode_instructions).unwrap() {
            assert_eq!(value, 0x123456789012);
            assert_eq!(name, b"dinosaur__");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_decode_i8() {
        let mut decode_instructions = DecodeInstructions::default();
        decode_instructions.data_types[1] = DataType::Signed;
        decode_instructions.fields[1] = *b"dinosaur__";

        let mut input = [0u8; 24];
        input[1] = (-24i8).to_le_bytes()[0];

        if let DecodedInput::I8 { value, name } = input.decode(1, &decode_instructions).unwrap() {
            assert_eq!(value, -24);
            assert_eq!(name, b"dinosaur__");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_decode_i16() {
        let mut decode_instructions = DecodeInstructions::default();
        decode_instructions.data_sizes[1] = DataSize::Two;
        decode_instructions.data_types[1] = DataType::Signed;
        decode_instructions.fields[1] = *b"dinosaur__";

        let mut input = [0u8; 24];
        input[1..3].copy_from_slice(&(-0x1234i16).to_le_bytes());

        if let DecodedInput::I16 { value, name } = input.decode(1, &decode_instructions).unwrap() {
            assert_eq!(value, -0x1234);
            assert_eq!(name, b"dinosaur__");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_decode_i32() {
        let mut decode_instructions = DecodeInstructions::default();
        decode_instructions.data_sizes[1] = DataSize::Four;
        decode_instructions.data_types[1] = DataType::Signed;
        decode_instructions.fields[1] = *b"dinosaur__";

        let mut input = [0u8; 24];
        input[1..5].copy_from_slice(&(-0x12345678i32).to_le_bytes());

        if let DecodedInput::I32 { value, name } = input.decode(1, &decode_instructions).unwrap() {
            assert_eq!(value, -0x12345678);
            assert_eq!(name, b"dinosaur__");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_decode_i64() {
        let mut decode_instructions = DecodeInstructions::default();
        decode_instructions.data_sizes[1] = DataSize::Eight;
        decode_instructions.data_types[1] = DataType::Signed;
        decode_instructions.fields[1] = *b"dinosaur__";

        let mut input = [0u8; 24];
        input[1..9].copy_from_slice(&(-0x123456789012i64).to_le_bytes());

        if let DecodedInput::I64 { value, name } = input.decode(1, &decode_instructions).unwrap() {
            assert_eq!(value, -0x123456789012);
            assert_eq!(name, b"dinosaur__");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_decode_f32() {
        let mut decode_instructions = DecodeInstructions::default();
        decode_instructions.data_sizes[1] = DataSize::Four;
        decode_instructions.data_types[1] = DataType::Floating;
        decode_instructions.fields[1] = *b"dinosaur__";

        let mut input = [0u8; 24];
        input[1..5].copy_from_slice(&(9.25f32).to_le_bytes());

        if let DecodedInput::F32 { value, name } = input.decode(1, &decode_instructions).unwrap() {
            assert_eq!(value, 9.25);
            assert_eq!(name, b"dinosaur__");
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_decode_f64() {
        let mut decode_instructions = DecodeInstructions::default();
        decode_instructions.data_sizes[1] = DataSize::Eight;
        decode_instructions.data_types[1] = DataType::Floating;
        decode_instructions.fields[1] = *b"dinosaur__";

        let mut input = [0u8; 24];
        input[1..9].copy_from_slice(&(125.75f64).to_le_bytes());

        if let DecodedInput::F64 { value, name } = input.decode(1, &decode_instructions).unwrap() {
            assert_eq!(value, 125.75);
            assert_eq!(name, b"dinosaur__");
        } else {
            assert!(false);
        }
    }
}