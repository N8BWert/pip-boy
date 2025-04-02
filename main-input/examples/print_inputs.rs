//!
//! Test program to print the current inputs when inputs are registered
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
    use embedded_hal::digital::InputPin;
    use rp_pico::{hal::{clocks::init_clocks_and_plls, Sio, Watchdog}, Pins};
    use fugit::{ExtU32, Instant};

    use rtic_monotonics::{rp2040::prelude::*, rp2040_timer_monotonic};

    use main_input::peripherals::*;
    use main_input::{check_three_input, check_four_input, INPUT_UPDATE_DELAY_MS};

    rp2040_timer_monotonic!(Mono);

    #[shared]
    struct Shared {

    }

    #[local]
    struct Local {
        switch: Switch,
        b1: B1,
        b2: B2,
        b3: B3,
        b4: B4,
        b5: B5,
        b6: B6,
        b7: B7,
        b8: B8,
        b9: B9,
        bback: BBack,
        b0: B0,
        bfront: BFront,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local) {
        Mono::start(ctx.device.TIMER, &mut ctx.device.RESETS);

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
            &mut watchdog
        )
        .ok()
        .unwrap();

        (
            Shared {

            },
            Local {
                switch: pins.gpio10.into_pull_down_input(),
                b1: pins.gpio11.into_pull_up_input(),
                b2: pins.gpio12.into_pull_up_input(),
                b3: pins.gpio13.into_pull_up_input(),
                b4: pins.gpio14.into_pull_up_input(),
                b5: pins.gpio15.into_pull_up_input(),
                b6: pins.gpio16.into_pull_up_input(),
                b7: pins.gpio17.into_pull_up_input(),
                b8: pins.gpio18.into_pull_up_input(),
                b9: pins.gpio19.into_pull_up_input(),
                bback: pins.gpio20.into_pull_up_input(),
                b0: pins.gpio21.into_pull_up_input(),
                bfront: pins.gpio22.into_pull_up_input(),
            }
        )
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[task(priority = 1)]
    /// Schedule and dispatch the tasks to update the current input state and read the input state from the
    /// connected buttons and switch
    async fn dispatch_input_tasks(_ctx: dispatch_input_tasks::Context) {
        loop {
            let now  = Mono::now();
            let next_read = now + INPUT_UPDATE_DELAY_MS.millis();

            if update_inputs::spawn().is_err() {
                defmt::error!("Update Inputs was Already Running");
            }

            Mono::delay_until(next_read).await;
        }
    }

    #[task(
        local = [
            switch,
            last_switch_value: bool = false,
            last_switch_time: Option<Instant<u64, 1, 1_000_000>> = None,
            b1,
            last_b1_value: bool = true,
            last_b1_time: Option<Instant<u64, 1, 1_000_000>> = None,
            b2,
            last_b2_value: bool = true,
            last_b2_click: u8 = 0,
            last_b2_time: Option<Instant<u64, 1, 1_000_000>> = None,
            b3,
            last_b3_value: bool = true,
            last_b3_click: u8 = 0,
            last_b3_time: Option<Instant<u64, 1, 1_000_000>> = None,
            b4,
            last_b4_value: bool = true,
            last_b4_click: u8 = 0,
            last_b4_time: Option<Instant<u64, 1, 1_000_000>> = None,
            b5,
            last_b5_value: bool = true,
            last_b5_click: u8 = 0,
            last_b5_time: Option<Instant<u64, 1, 1_000_000>> = None,
            b6,
            last_b6_value: bool = true,
            last_b6_click: u8 = 0,
            last_b6_time: Option<Instant<u64, 1, 1_000_000>> = None,
            b7,
            last_b7_value: bool = true,
            last_b7_click: u8 = 0,
            last_b7_time: Option<Instant<u64, 1, 1_000_000>> = None,
            b8,
            last_b8_value: bool = true,
            last_b8_click: u8 = 0,
            last_b8_time: Option<Instant<u64, 1, 1_000_000>> = None,
            b9,
            last_b9_value: bool = true,
            last_b9_click: u8 = 0,
            last_b9_time: Option<Instant<u64, 1, 1_000_000>> = None,
            bback,
            last_back_value: bool = true,
            last_back_time: Option<Instant<u64, 1, 1_000_000>> = None,
            b0,
            last_b0_value: bool = true,
            last_b0_time: Option<Instant<u64, 1, 1_000_000>> = None,
            bfront,
            last_front_value: bool = true,
            last_front_time: Option<Instant<u64, 1, 1_000_000>> = None,
        ],
        priority = 1
    )]
    /// Check the external inputs and the inputs connected to this module and replace the current input
    /// state with the new input state
    async fn update_inputs(ctx: update_inputs::Context) {
        let now = Mono::now();

        // Update inputs based on pressed buttons and pressed button states
        if ctx.local.switch.is_high().unwrap() {
            defmt::info!("SHIFT");
        }

        if ctx.local.b1.is_high().unwrap() {
            defmt::info!("1");
        }

        let b2_high = ctx.local.b2.is_high().unwrap();
        if b2_high {
            let (a, b, _c) = check_three_input(now, *ctx.local.last_b2_time, ctx.local.last_b2_click);
            defmt::info!("2 - {} - UP", if a { "A" } else if b { "B" } else { "C" });
            *ctx.local.last_b2_time = Some(now);
        } else if *ctx.local.last_b2_value {
            *ctx.local.last_b2_time = Some(now);
        }
        *ctx.local.last_b2_value = b2_high;

        let b3_high = ctx.local.b3.is_high().unwrap();
        if b3_high {
            let (d, e, _f) = check_three_input(now, *ctx.local.last_b3_time, ctx.local.last_b2_click);
            defmt::info!("3 - {}", if d { "D" } else if e { "E" } else { "F" });
            *ctx.local.last_b3_time = Some(now);
        } else if *ctx.local.last_b3_value {
            *ctx.local.last_b3_time = Some(now);
        }
        *ctx.local.last_b3_value = b3_high;

        let b4_high = ctx.local.b4.is_high().unwrap();
        if b4_high {
            let (g, h, _i) = check_three_input(now, *ctx.local.last_b4_time, ctx.local.last_b4_click);
            defmt::info!("4 - {} - LEFT", if g { "G" } else if h { "H" } else { "I" });
            *ctx.local.last_b4_time = Some(now);
        } else if *ctx.local.last_b4_value {
            *ctx.local.last_b4_time = Some(now);
        }
        *ctx.local.last_b4_value = b4_high;

        let b5_high = ctx.local.b5.is_high().unwrap();
        if b5_high {
            let (j, k, _l) = check_three_input(now, *ctx.local.last_b5_time, ctx.local.last_b5_click);
            defmt::info!("5 - {}", if j { "J" } else if k { "K" } else { "L" });
            *ctx.local.last_b5_time = Some(now);
        } else if *ctx.local.last_b5_value {
            *ctx.local.last_b5_time = Some(now);
        }
        *ctx.local.last_b5_value = b5_high;

        let b6_high = ctx.local.b6.is_high().unwrap();
        if b6_high {
            let (m, n, _o) = check_three_input(now, *ctx.local.last_b6_time, ctx.local.last_b6_click);
            defmt::info!("6 - {} - RIGHT", if m { "M" } else if n { "N" } else { "O" });
            *ctx.local.last_b6_time = Some(now);
        } else if *ctx.local.last_b6_value {
            *ctx.local.last_b6_time = Some(now);
        }
        *ctx.local.last_b6_value = b6_high;

        let b7_high = ctx.local.b7.is_high().unwrap();
        if b7_high {
            let (p, q, r, _s) = check_four_input(now, *ctx.local.last_b7_time, ctx.local.last_b7_click);
            defmt::info!("7 - {}", if p { "P" } else if q { "Q" } else if r { "R" } else { "S" });
            *ctx.local.last_b7_time = Some(now);
        } else if *ctx.local.last_b7_value {
            *ctx.local.last_b7_time = Some(now);
        }
        *ctx.local.last_b7_value = b7_high;

        let b8_high = ctx.local.b8.is_high().unwrap();
        if b8_high {
            let (t, u, _v) = check_three_input(now, *ctx.local.last_b8_time, ctx.local.last_b8_click);
            defmt::info!("8 - {} - DOWN", if t { "T" } else if u { "U" } else { "V" });
            *ctx.local.last_b8_time = Some(now);
        } else if *ctx.local.last_b8_value {
            *ctx.local.last_b8_time = Some(now);
        }
        *ctx.local.last_b8_value = b8_high;

        let b9_high = ctx.local.b9.is_high().unwrap();
        if b9_high {
            let (w, x, y, _z) = check_four_input(now, *ctx.local.last_b9_time, ctx.local.last_b9_click);
            defmt::info!("9 - {}", if w { "W" } else if x { "X" } else if y { "Y" } else { "Z" });
            *ctx.local.last_b9_time = Some(now);
        } else {
            *ctx.local.last_b9_time = Some(now);
        }
        *ctx.local.last_b9_value = b9_high;

        if ctx.local.bback.is_high().unwrap() {
            defmt::info!("<-");
        }

        if ctx.local.b0.is_high().unwrap() {
            defmt::info!("0");
        }

        if ctx.local.bfront.is_high().unwrap() {
            defmt::info!("ENTER");
        }
    }
}