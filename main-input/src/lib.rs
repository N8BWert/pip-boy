//!
//! Library Definitions for the Main Input Module
//!

#![no_std]

pub mod peripherals;

use fugit::{ExtU32, Instant};

/// The amount of time between updating the input state
pub const INPUT_UPDATE_DELAY_MS: u32 = 10;
/// The amount of time between button presses to consider the press as modulating the key value
const SEQUENCE_DELAY_MS: u32 = 500;

/// From the outputs of a pin, check which of the three inputs should be selected
pub fn check_three_input(
    now: Instant<u64, 1, 1_000_000>,
    last_time: Option<Instant<u64, 1, 1_000_000>>,
    last_click: &mut u8,
) -> (bool, bool, bool) {
    match last_time {
        Some(time) => {
            if now - time < SEQUENCE_DELAY_MS.millis::<1, 1_000_000>() {
                *last_click += 1;
                match *last_click % 3 {
                    0 => (true, false, false),
                    1 => (false, true, false),
                    _ => (false, false, true),
                }
            } else {
                *last_click = 0;
                (true, false, false)
            }
        },
        None => (true, false, false),
    }
}

/// From the outputs of a pin, check which of the four inputs should be selected
pub fn check_four_input(
    now: Instant<u64, 1, 1_000_000>,
    last_time: Option<Instant<u64, 1, 1_000_000>>,
    last_click: &mut u8,
) -> (bool, bool, bool, bool) {
    match last_time {
        Some(time) => {
            if now - time < SEQUENCE_DELAY_MS.millis::<1, 1_000_000>() {
                *last_click += 1;
                match *last_click % 4 {
                    0 => (true, false, false, false),
                    1 => (false, true, false, false),
                    2 => (false, false, true, false),
                    _ => (false, false, false, true),
                }
            } else {
                *last_click = 0;
                (true, false, false, false)
            }
        },
        None => (true, false, false, false),
    }
}