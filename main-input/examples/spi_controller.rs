//!
//! Test program to ensure the spi controller is functioning as desired
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
    use core::cell::RefCell;

    use common::{input::{Input, InputRequest}, prelude::Unpack};
    use critical_section::Mutex;
    use embedded_hal::{digital::InputPin, spi::{SpiDevice, MODE_0}};
    use rp_pico::{hal::{clocks::init_clocks_and_plls, gpio::{FunctionSpi, Interrupt}, Sio, Spi, Watchdog}, Pins};
    use fugit::{RateExtU32, ExtU32};

    use rtic_monotonics::{rp2040::prelude::*, rp2040_timer_monotonic};

    use embedded_hal_bus::spi::CriticalSectionDevice;

    use main_input::peripherals::*;

    rp2040_timer_monotonic!(Mono);

    /// Static Variable Holding Spi Bus 0.  This should only every be set and referred to in `init`. Elsewhere, use the actual spi device
    static mut SPI_BUS: Option<SpiBus0> = None;

    #[shared]
    struct Shared {
        /// true if extension 1 is enabled
        ext1_enabled: bool,
        /// Pin indicating if extension 1 is enabled
        en_ext1: EnExt1,
        /// The spi connected to extension 1
        ext1_spi: Ext1Spi,

        /// true if extension 2 is enabled
        ext2_enabled: bool,
        /// Pin indicating if extension 2 is enabled
        en_ext2: EnExt2,
        /// The spi connected to extension 2
        ext2_spi: Ext2Spi,
    }

    #[local]
    struct Local {

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

        let spi_device = ctx.device.SPI0;
        let spi_pin_layout = (
            pins.gpio3.into_function::<FunctionSpi>(),
            pins.gpio4.into_function::<FunctionSpi>(),
            pins.gpio2.into_function::<FunctionSpi>(),
        );

        let bus = Mutex::new(RefCell::new(
            Spi::<_, _, _, 8>::new(spi_device, spi_pin_layout)
                .init(&mut ctx.device.RESETS, 125_000_000u32.Hz(), 16_000_000u32.Hz(), MODE_0)
        ));

        #[allow(static_mut_refs)]
        unsafe { SPI_BUS.replace(bus); }
        let cs1 = pins.gpio5.into_push_pull_output();
        let cs2 = pins.gpio9.into_push_pull_output();

        #[allow(static_mut_refs)]
        let ext1_spi = CriticalSectionDevice::new_no_delay(unsafe { SPI_BUS.as_ref().unwrap() }, cs1).unwrap();
        #[allow(static_mut_refs)]
        let ext2_spi= CriticalSectionDevice::new_no_delay(unsafe { SPI_BUS.as_ref().unwrap() }, cs2).unwrap();

        let mut en_ext1 = pins.gpio0.into_pull_down_input();
        let ext1_enabled = en_ext1.is_high().unwrap();
        if ext1_enabled {
            en_ext1.set_interrupt_enabled(Interrupt::EdgeLow, true);
        } else {
            en_ext1.set_interrupt_enabled(Interrupt::EdgeHigh, true);
        }

        let mut en_ext2 = pins.gpio1.into_pull_down_input();
        let ext2_enabled = en_ext2.is_high().unwrap();
        if ext2_enabled {
            en_ext2.set_interrupt_enabled(Interrupt::EdgeLow, true);
        } else {
            en_ext2.set_interrupt_enabled(Interrupt::EdgeHigh, true);
        }

        (
            Shared {
                ext1_enabled,
                ext2_enabled,
                en_ext1,
                en_ext2,
                ext1_spi,
                ext2_spi,
            },
            Local {

            }
        )
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            cortex_m::asm::wfi();
        }
    }

    #[task(
        priority = 1
    )]
    /// Schedule and dispatch the tasks to update the current input state and read the input state from the
    /// connected buttons and switch
    async fn dispatch_input_tasks(_ctx: dispatch_input_tasks::Context) {
        loop {
            let now  = Mono::now();
            let next_read = now + 1_000u32.millis();

            if update_inputs::spawn().is_err() {
                defmt::error!("Update Inputs was Already Running");
            }

            Mono::delay_until(next_read).await;
        }
    }

    #[task(
        shared = [
            ext1_enabled,
            ext2_enabled,
            ext1_spi,
            ext2_spi,
        ],
        priority = 1
    )]
    /// Check the inputs from the spi
    async fn update_inputs(mut ctx: update_inputs::Context) {
        if ctx.shared.ext1_enabled.lock(|ext1_enabled| *ext1_enabled) {
            let mut buffer = [0u8; 71];
            ctx.shared.ext1_spi.lock(|spi| {
                spi.write(&[InputRequest::FullInput as u8]).unwrap();
                spi.transfer_in_place(&mut buffer).unwrap();
            });
            let input = Input::unpack(&buffer).unwrap();
            defmt::info!("Keypad 1: {:?}", input.keypad);
        } else {
            defmt::info!("EXT1 Not Connected");
        }

        if ctx.shared.ext2_enabled.lock(|ext2_enabled| *ext2_enabled) {
            let mut buffer = [0u8; 71];
            ctx.shared.ext2_spi.lock(|spi| {
                spi.write(&[InputRequest::FullInput as u8]).unwrap();
                spi.transfer_in_place(&mut buffer).unwrap();
            });
            let input = Input::unpack(&buffer).unwrap();
            defmt::info!("Keypad 2: {:?}", input.keypad);
        } else {
            defmt::info!("EXT2 Not Connected");
        }
    }

    #[task(
        shared = [
            ext1_enabled,
            ext2_enabled,
            en_ext1,
            en_ext2,
            ext1_spi,
            ext2_spi,
        ],
        priority = 1,
        binds = IO_IRQ_BANK0
    )]
    /// Interrupt Called Whenever an Extension Module is Connected or Disconnected
    fn power_interrupt(ctx: power_interrupt::Context) {
        let (new_one, new_two) = (
            ctx.shared.ext1_enabled,
            ctx.shared.ext2_enabled,
            ctx.shared.en_ext1,
            ctx.shared.en_ext2
        ).lock(|ext1_enabled, ext2_enabled, en_ext1, en_ext2| {
            // Check if extension 1 has changed connection
            let ext1 = en_ext1.is_high().unwrap();
            let mut new_one = false;
            if ext1 != *ext1_enabled {
                *ext1_enabled = ext1;
                if ext1 {
                    en_ext1.set_interrupt_enabled(Interrupt::EdgeHigh, false);
                    en_ext1.set_interrupt_enabled(Interrupt::EdgeLow, true);
                    new_one = true;
                } else {
                    en_ext1.set_interrupt_enabled(Interrupt::EdgeLow, false);
                    en_ext1.set_interrupt_enabled(Interrupt::EdgeHigh, true);
                }
            }

            // Check if extension 2 has changed connection
            let ext2 = en_ext2.is_high().unwrap();
            let mut new_two = false;
            if ext2 != *ext2_enabled {
                if ext2 {
                    en_ext2.set_interrupt_enabled(Interrupt::EdgeHigh, false);
                    en_ext2.set_interrupt_enabled(Interrupt::EdgeLow, true);
                    new_two = true;
                } else {
                    en_ext2.set_interrupt_enabled(Interrupt::EdgeLow, false);
                    en_ext2.set_interrupt_enabled(Interrupt::EdgeHigh, true);
                }
            }

            (new_one, new_two)
        });

        if new_one {
            defmt::info!("EXTENSION 1 CONNECTED");
        }

        if new_two {
            defmt::info!("EXTENSION 2 CONNECTED");
        }
    }
}