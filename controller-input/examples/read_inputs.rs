//!
//! Test program that prints the inputs registered by the controller
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
    use common::input::Input;
    use rp_pico::{hal::{adc::AdcPin, clocks::init_clocks_and_plls, timer::{Alarm, Alarm0}, Adc, Sio, Timer, Watchdog}, Pins};
    use fugit::ExtU32;
    use embedded_hal_0_2::{adc::OneShot, digital::v2::InputPin};

    use controller_input::{peripherals::*, READ_DELAY_US};

    #[shared]
    struct Shared {
        // The current input state of the controller
        input: Input
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

        let mut timer = Timer::new(ctx.device.TIMER, &mut ctx.device.RESETS, &clocks);
        let mut alarm0 = timer.alarm_0().unwrap();
        alarm0.schedule(READ_DELAY_US.micros()).unwrap();

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
            }
        )
    }

    #[task(
        shared = [input],
        local = [x, y, a, b, adc, alarm, iteration: u32 = 0],
        priority = 1,
        binds = TIMER_IRQ_0
    )]
    fn read_pins(mut ctx: read_pins::Context) {
        ctx.local.alarm.clear_interrupt();
        let x: u16 = ctx.local.adc.read(ctx.local.x).unwrap();
        let y: u16 = ctx.local.adc.read(ctx.local.y).unwrap();
        let a = ctx.local.a.is_high().unwrap();
        let b = ctx.local.b.is_high().unwrap();

        let input = ctx.shared.input.lock(|input| {
            input.keypad.a = a;
            input.keypad.b = b;
            input.analog.a0 = x;
            input.analog.a1 = y;

            if *ctx.local.iteration % 100 == 0 {
                Some(input.clone())
            } else {
                None
            }
        });

        if let Some(input) = input {
            defmt::info!(
                "A: {}, B: {}, X: {}, Y: {}",
                input.keypad.a,
                input.keypad.b,
                input.analog.a0,
                input.analog.a1,
            );
        }

        ctx.local.alarm.schedule(READ_DELAY_US.micros()).unwrap();
    }
}