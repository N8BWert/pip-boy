//!
//! Test program to test that the spi slave implementation works
//! 

#![no_std]
#![no_main]

use defmt_rtt as _;
use panic_probe as _;

#[rtic::app(
    device = rp_pico::hal::pac,
    peripherals = true,
    dispatchers = [SW0_IRQ, SW1_IRQ]
)]
mod app {
    use common::{input::{Input, InputRequest}, prelude::{AnalogInputsBuilder, AuxiliaryBuilder, KeypadBuilder, NumpadBuilder, Pack}};
    use embedded_hal::spi::MODE_0;
    use rp_pico::{hal::{self, clocks::init_clocks_and_plls, gpio::FunctionSpi, spi::FrameFormat, Sio, Spi, Watchdog}, Pins};
    use embedded_hal_nb::spi::FullDuplex;

    use controller_input::peripherals::*;

    #[shared]
    struct Shared {
        // The current input state of the controller
        input: Input,
    }

    #[local]
    struct Local {
        // The spi line coming into the controller input
        spi_line: SpiLine,
        // The chip select for the controller input spi line
        csn: CSn,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local) {
        let sio = Sio::new(ctx.device.SIO);
        let pins = Pins::new(
            ctx.device.IO_BANK0,
            ctx.device.PADS_BANK0,
            sio.gpio_bank0,
            &mut ctx.device.RESETS
        );

        let mut watchdog = Watchdog::new(ctx.device.WATCHDOG);
        let _clocks = init_clocks_and_plls(
            12_000_000u32,
            ctx.device.XOSC,
            ctx.device.CLOCKS,
            ctx.device.PLL_SYS,
            ctx.device.PLL_USB,
            &mut ctx.device.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let spi = Spi::<_, _, _, 8>::new(
            ctx.device.SPI0,
            (
                pins.gpio3.into_function::<FunctionSpi>(),
                pins.gpio4.into_function::<FunctionSpi>(),
                pins.gpio2.into_function::<FunctionSpi>(),
            )
        );
        let spi_slave = spi.init_slave(
            &mut ctx.device.RESETS,
            FrameFormat::MotorolaSpi(MODE_0)
        );
        let csn = pins.gpio5.into_pull_up_input();
        // csn.set_interrupt_enabled(Interrupt::EdgeLow, true);

        hal::pac::NVIC::unpend(hal::pac::Interrupt::SPI0_IRQ);
        unsafe {
            hal::pac::NVIC::unmask(hal::pac::Interrupt::SPI0_IRQ);
        }

        (
            Shared {
                input: Input {
                    numpad: NumpadBuilder::default()
                        .build()
                        .unwrap(),
                    keypad: KeypadBuilder::default()
                        .a(true)
                        .build()
                        .unwrap(),
                    auxiliary: AuxiliaryBuilder::default()
                        .build()
                        .unwrap(),
                    analog: AnalogInputsBuilder::default()
                        .a0(200)
                        .a1(1600)
                        .build()
                        .unwrap(),
                    other_input_one: [0u8; 24],
                    other_input_two: [0u8; 24],
                },
            },
            Local {
                spi_line: spi_slave,
                csn,
            }
        )
    }

    #[task(
        shared = [input],
        local = [spi_line, csn],
        priority = 2,
        binds = SPI0_IRQ
    )]
    fn relay_inputs(mut ctx: relay_inputs::Context) {
        ctx.shared.input.lock(|input| {
            let instruction = InputRequest::from(ctx.local.spi_line.read().unwrap());
            match instruction {
                InputRequest::FullInput => {
                    let mut buffer = [0u8; 71];
                    input.pack(&mut buffer).unwrap();
                    for byte in buffer {
                        ctx.local.spi_line.write(byte).unwrap();
                    }
                },
                InputRequest::Numpad => {
                    let mut buffer = [0u8; 2];
                    input.pack(&mut buffer).unwrap();
                    for byte in buffer {
                        ctx.local.spi_line.write(byte).unwrap();
                    }
                },
                InputRequest::Keypad => {
                    let mut buffer = [0u8; 4];
                    input.pack(&mut buffer).unwrap();
                    for byte in buffer {
                        ctx.local.spi_line.write(byte).unwrap();
                    }
                },
                InputRequest::Auxiliary => {
                    let mut buffer = [0u8; 4];
                    input.pack(&mut buffer).unwrap();
                    for byte in buffer {
                        ctx.local.spi_line.write(byte).unwrap();
                    }
                },
                InputRequest::Analog => {
                    let mut buffer = [0u8; 12];
                    input.pack(&mut buffer).unwrap();
                    for byte in buffer {
                        ctx.local.spi_line.write(byte).unwrap();
                    }
                },
                _ => (),
            }
        });
    }
}