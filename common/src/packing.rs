//!
//! Packing and unpacking traits
//! 

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Error from packing data
pub enum PackingError {
    /// The buffer size was not large enought to accomidate the data
    InvalidBufferSize,
}

/// Trait for packing data into a buffer for transmission over some protocol
pub trait Pack {
    /// Pack the data into a given buffer slice
    fn pack(self, buffer: &mut [u8]) -> Result<(), PackingError>;
}

/// Trait for unpacking data from a buffer
pub trait Unpack {
    /// Unpack the data from a given buffer
    fn unpack(buffer: &[u8]) -> Result<Self, PackingError> where Self: Sized;
}