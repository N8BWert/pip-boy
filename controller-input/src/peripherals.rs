//!
//! Peripheral Definitions for each device connected to the controller input module
//! 

use rp_pico::{hal::{adc::AdcPin, gpio::{bank0::{Gpio2, Gpio26, Gpio27, Gpio3, Gpio4, Gpio5, Gpio6, Gpio7}, FunctionSio, FunctionSpi, Pin, PullDown, PullNone, PullUp, SioInput, SioOutput}, spi::Enabled, Spi}, pac::SPI0};

/// The x-direction adc input
pub type X = AdcPin<Pin<Gpio26, FunctionSio<SioInput>, PullNone>>;
/// The y-direction adc input
pub type Y = AdcPin<Pin<Gpio27, FunctionSio<SioInput>, PullNone>>;
/// The first input from the controller
pub type A = Pin<Gpio6, FunctionSio<SioOutput>, PullDown>;
/// The second input from the controller
pub type B = Pin<Gpio7, FunctionSio<SioOutput>, PullDown>;

/// The spi line from the main input module
pub type SpiLine = Spi<Enabled, SPI0, (Pin<Gpio3, FunctionSpi, PullDown>, Pin<Gpio4, FunctionSpi, PullDown>, Pin<Gpio2, FunctionSpi, PullDown>)>;
/// The chip select pin for the spi
pub type CSn = Pin<Gpio5, FunctionSio<SioInput>, PullUp>;