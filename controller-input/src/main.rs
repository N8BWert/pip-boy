//!
//! Main Program Running on the Pip Boy's Controller Input Module
//! 
//! The Controller Input Module takes inputs from a controller and relays them to the
//! main input module
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
    use common::{input::{Input, InputRequest}, prelude::Pack};
    use embedded_hal::spi::MODE_0;
    use rp_pico::{hal::{self, adc::AdcPin, clocks::init_clocks_and_plls, gpio::FunctionSpi, spi::FrameFormat, timer::{Alarm, Alarm0}, Adc, Sio, Spi, Timer, Watchdog}, Pins};
    use fugit::ExtU32;
    use embedded_hal_0_2::{adc::OneShot, digital::v2::InputPin};
    use embedded_hal_nb::spi::FullDuplex;

    use controller_input::{peripherals::*, READ_DELAY_US};

    #[shared]
    struct Shared {
        // The current input state of the controller
        input: Input,
    }

    #[local]
    struct Local {
        // The x axis pwm input
        x: X,
        // The y axis pwm input
        y: Y,
        // The a button
        a: A,
        // The b button
        b: B,
        // The adc peripheral to read adc values
        adc: Adc,
        // The alarm to schedule input updates
        alarm: Alarm0,

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
        let clocks = init_clocks_and_plls(
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

        let adc = Adc::new(ctx.device.ADC, &mut ctx.device.RESETS);

        let x  = AdcPin::new(pins.gpio26.into_floating_input()).unwrap();
        let y = AdcPin::new(pins.gpio27.into_floating_input()).unwrap();
        let a = pins.gpio6.into_push_pull_output();
        let b = pins.gpio7.into_push_pull_output();

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

        let mut timer = Timer::new(ctx.device.TIMER, &mut ctx.device.RESETS, &clocks);
        let mut alarm0 = timer.alarm_0().unwrap();
        alarm0.schedule(READ_DELAY_US.micros()).unwrap();

        hal::pac::NVIC::unpend(hal::pac::Interrupt::SPI0_IRQ);
        unsafe {
            hal::pac::NVIC::unmask(hal::pac::Interrupt::SPI0_IRQ);
        }

        (
            Shared {
                input: Input::default(),
            },
            Local {
                x,
                y,
                a,
                b,
                adc,
                alarm: alarm0,
                spi_line: spi_slave,
                csn,
            }
        )
    }

    #[task(
        shared = [input],
        local = [x, y, a, b, adc, alarm],
        priority = 1,
        binds = TIMER_IRQ_0
    )]
    /// Read the current inputs from the peripherals
    fn read_pins(mut ctx: read_pins::Context) {
        ctx.local.alarm.clear_interrupt();
        let x: u16 = ctx.local.adc.read(ctx.local.x).unwrap();
        let y: u16 = ctx.local.adc.read(ctx.local.y).unwrap();
        let a = ctx.local.a.is_high().unwrap();
        let b = ctx.local.b.is_high().unwrap();

        ctx.shared.input.lock(|input| {
            input.keypad.a = a;
            input.keypad.b = b;
            input.analog.a0 = x;
            input.analog.a1 = y;
        });

        ctx.local.alarm.schedule(READ_DELAY_US.micros()).unwrap();
    }

    #[task(
        shared = [input],
        local = [spi_line, csn],
        priority = 2,
        binds = SPI0_IRQ
    )]
    /// Return the current input state of the controller
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