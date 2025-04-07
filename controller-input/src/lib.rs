//!
//! Library Definitions for the Controller Input Module
//! 

#![no_std]

pub mod peripherals;

/// The amount of time between subsequent readings of the inputs
pub const READ_DELAY_US: u32 = 1_000;
