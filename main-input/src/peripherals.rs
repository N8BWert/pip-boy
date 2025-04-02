//!
//! Peripheral Definitions for each device connected to the main input module
//!

use critical_section::Mutex;
use core::cell::RefCell;

use rp_pico::{
    hal::{
        gpio::{
            bank0::{
                Gpio0, Gpio1, Gpio10, Gpio11, Gpio12, Gpio13, Gpio14, Gpio15, Gpio16, Gpio17, Gpio18, Gpio19, Gpio2, Gpio20, Gpio21, Gpio22, Gpio3, Gpio4, Gpio5, Gpio6, Gpio7, Gpio9
            }, FunctionI2c, FunctionSio, FunctionSpi, Pin, PullDown, PullUp, SioInput, SioOutput
        }, i2c::Peripheral, spi::Enabled, Spi, I2C
    },
    pac::{I2C1, SPI0},
};

use embedded_hal_bus::spi::{CriticalSectionDevice, NoDelay};

/// SPI0
type Spi0 = Spi<Enabled, SPI0, (Pin<Gpio3, FunctionSpi, PullDown>, Pin<Gpio4, FunctionSpi, PullDown>, Pin<Gpio2, FunctionSpi, PullDown>)>;
/// A bus for SPI0
pub type SpiBus0 = Mutex<RefCell<Spi0>>;

/// The enable pin for enabling extension 1
pub type EnExt1 = Pin<Gpio0, FunctionSio<SioInput>, PullDown>;
/// The spi device connected to extension 1
pub type Ext1Spi = CriticalSectionDevice<'static, Spi0, Pin<Gpio5, FunctionSio<SioOutput>, PullDown>, NoDelay>;

/// The enable pin for enabling extension 2
pub type EnExt2 = Pin<Gpio1, FunctionSio<SioInput>, PullDown>;
/// The spi device connected to extension 2
pub type Ext2Spi = CriticalSectionDevice<'static, Spi0, Pin<Gpio9, FunctionSio<SioOutput>, PullDown>, NoDelay>;

/// The i2c peripheral the programming modules use to communicate with the main input module
pub type ProgramI2C = I2C<I2C1, (Pin<Gpio6, FunctionI2c, PullUp>, Pin<Gpio7, FunctionI2c, PullUp>), Peripheral>;

/// The switch
pub type Switch = Pin<Gpio10, FunctionSio<SioInput>, PullDown>;

/// The first button on the keypad
pub type B1 = Pin<Gpio11, FunctionSio<SioInput>, PullUp>;
/// The second button on the keypad
pub type B2 = Pin<Gpio12, FunctionSio<SioInput>, PullUp>;
/// The third button on the keypad
pub type B3 = Pin<Gpio13, FunctionSio<SioInput>, PullUp>;
/// The fourth button on the keypad
pub type B4 = Pin<Gpio14, FunctionSio<SioInput>, PullUp>;
/// The fifth button on the keypad
pub type B5 = Pin<Gpio15, FunctionSio<SioInput>, PullUp>;
/// The sixth button on the keypad
pub type B6 = Pin<Gpio16, FunctionSio<SioInput>, PullUp>;
/// The seventh button on the keypad
pub type B7 = Pin<Gpio17, FunctionSio<SioInput>, PullUp>;
/// The eighth button on the keypad
pub type B8 = Pin<Gpio18, FunctionSio<SioInput>, PullUp>;
/// The ninth button the keypad
pub type B9 = Pin<Gpio19, FunctionSio<SioInput>, PullUp>;
/// The back button on the keypad
pub type BBack = Pin<Gpio20, FunctionSio<SioInput>, PullUp>;
/// The zero button on the keypad
pub type B0 = Pin<Gpio21, FunctionSio<SioInput>, PullUp>;
/// The forward button on the keypad
pub type BFront = Pin<Gpio22, FunctionSio<SioInput>, PullUp>;
