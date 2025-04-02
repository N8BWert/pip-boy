//!
//! Ease of use prelude for the pip-boy common library
//! 

pub use crate::packing::{Pack, Unpack, PackingError};
pub use crate::input::{
    Input, InputBuilder,
    analog::{AnalogInputs, AnalogInputsBuilder},
    auxiliary::{Auxiliary, AuxiliaryBuilder},
    keypad::{Keypad, KeypadBuilder},
    numpad::{Numpad, NumpadBuilder},
    other::{DataSize, DataType, DecodeInstructions, OtherInput},
};
