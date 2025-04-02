//!
//! Main Program Running on the Pip Boy's Main Input Module
//!
//! Basically, the main input module polls the added input modules for the current input state
//! and combines the inputs together to be polled by the running program module.
//!
//! By default, all inputs are assumed default so any input that is true will override the
//! default false values.
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

    use common::{input::{Input, InputRequest}, prelude::{Pack, Unpack}};
    use critical_section::Mutex;
    use embedded_hal::{digital::InputPin, spi::{SpiDevice, MODE_0}};
    use rp_pico::{hal::{self, clocks::init_clocks_and_plls, gpio::{FunctionSpi, Interrupt}, Sio, Spi, Watchdog, I2C}, pac::RESETS, Pins};
    use fugit::{RateExtU32, ExtU32, Instant};

    use rtic_monotonics::{rp2040::prelude::*, rp2040_timer_monotonic};

    use embedded_hal_bus::spi::CriticalSectionDevice;

    use main_input::peripherals::*;
    use main_input::{check_three_input, check_four_input, INPUT_UPDATE_DELAY_MS};

    rp2040_timer_monotonic!(Mono);

    /// Static Variable Holding Spi Bus 0.  This should only every be set and referred to in `init`. Elsewhere, use the actual spi device
    static mut SPI_BUS: Option<SpiBus0> = None;
    /// The address of this device on the i2c line
    static mut I2C_ADDRESS: u8 = 0;

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

        /// The i2c from the main programming modules
        program_i2c: Option<ProgramI2C>,

        /// The current combined input state of the modules
        input_state: Input,
        /// The resets device peripheral
        resets: RESETS,
        /// The decode instructions for extension 1
        ext1_decode_instructions: [u8; 248],
        /// The decode instructions for extension 2
        ext2_decode_instructions: [u8; 248],
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

        let program_i2c = I2C::new_peripheral_event_iterator(
            ctx.device.I2C1,
            pins.gpio6.reconfigure(),
            pins.gpio7.reconfigure(),
            &mut ctx.device.RESETS,
            0u8,
        );

        hal::pac::NVIC::unpend(hal::pac::Interrupt::I2C1_IRQ);
        unsafe {
            hal::pac::NVIC::unmask(hal::pac::Interrupt::I2C1_IRQ);
        }

        (
            Shared {
                ext1_enabled,
                ext2_enabled,
                en_ext1,
                en_ext2,
                ext1_spi,
                ext2_spi,
                program_i2c: Some(program_i2c),
                input_state: Input::default(),
                resets: ctx.device.RESETS,
                ext1_decode_instructions: [0u8; 248],
                ext2_decode_instructions: [0u8; 248],
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

    #[task(
        priority = 1
    )]
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
        shared = [
            input_state,
            ext1_enabled,
            ext2_enabled,
            ext1_spi,
            ext2_spi,
        ],
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
    async fn update_inputs(mut ctx: update_inputs::Context) {
        let mut next_input = Input::default();

        // Update extension 1 inputs
        if ctx.shared.ext1_enabled.lock(|ext1_enabled| *ext1_enabled) {
            let mut buffer = [0u8; 71];
            ctx.shared.ext1_spi.lock(|spi| {
                spi.write(&[InputRequest::FullInput as u8]).unwrap();
                spi.transfer_in_place(&mut buffer).unwrap();
            });
            next_input = Input::unpack(&buffer).unwrap();
        }

        // Update extension 2 inputs
        if ctx.shared.ext2_enabled.lock(|ext2_enabled| *ext2_enabled) {
            let mut buffer = [0u8; 71];
            ctx.shared.ext2_spi.lock(|spi| {
                spi.write(&[InputRequest::FullInput as u8]).unwrap();
                spi.transfer_in_place(&mut buffer).unwrap();
            });
            let input = Input::unpack(&buffer).unwrap();
            next_input |= input;
            next_input.analog.a3 = input.analog.a0;
            next_input.analog.a4 = input.analog.a1;
            next_input.analog.a5 = input.analog.a2;
            next_input.other_input_two = input.other_input_one;
        }

        let now = Mono::now();

        // Update inputs based on pressed buttons and pressed button states
        if ctx.local.switch.is_high().unwrap() {
            next_input.keypad.shift = true;
        }

        if ctx.local.b1.is_high().unwrap() {
            next_input.numpad.one = true;
        }

        let b2_high = ctx.local.b2.is_high().unwrap();
        if b2_high {
            next_input.numpad.two = true;

            (
                next_input.keypad.a,
                next_input.keypad.b,
                next_input.keypad.c
            ) = check_three_input(now, *ctx.local.last_b2_time, ctx.local.last_b2_click);
            *ctx.local.last_b2_time = Some(now);
        } else if *ctx.local.last_b2_value {
            *ctx.local.last_b2_time = Some(now);
        }
        *ctx.local.last_b2_value = b2_high;

        let b3_high = ctx.local.b3.is_high().unwrap();
        if b3_high {
            next_input.numpad.three = true;
            (
                next_input.keypad.d,
                next_input.keypad.e,
                next_input.keypad.f
            ) = check_three_input(now, *ctx.local.last_b3_time, ctx.local.last_b2_click);
            *ctx.local.last_b3_time = Some(now);
        } else if *ctx.local.last_b3_value {
            *ctx.local.last_b3_time = Some(now);
        }
        *ctx.local.last_b3_value = b3_high;

        let b4_high = ctx.local.b4.is_high().unwrap();
        if b4_high {
            next_input.numpad.four = true;
            (
                next_input.keypad.g,
                next_input.keypad.h,
                next_input.keypad.i
            ) = check_three_input(now, *ctx.local.last_b4_time, ctx.local.last_b4_click);
            *ctx.local.last_b4_time = Some(now);
        } else if *ctx.local.last_b4_value {
            *ctx.local.last_b4_time = Some(now);
        }
        *ctx.local.last_b4_value = b4_high;

        let b5_high = ctx.local.b5.is_high().unwrap();
        if b5_high {
            next_input.numpad.five = true;
            (
                next_input.keypad.j,
                next_input.keypad.k,
                next_input.keypad.l
            ) = check_three_input(now, *ctx.local.last_b5_time, ctx.local.last_b5_click);
            *ctx.local.last_b5_time = Some(now);
        } else if *ctx.local.last_b5_value {
            *ctx.local.last_b5_time = Some(now);
        }
        *ctx.local.last_b5_value = b5_high;

        let b6_high = ctx.local.b6.is_high().unwrap();
        if b6_high {
            next_input.numpad.six = true;
            (
                next_input.keypad.m,
                next_input.keypad.n,
                next_input.keypad.o,
            ) = check_three_input(now, *ctx.local.last_b6_time, ctx.local.last_b6_click);
            *ctx.local.last_b6_time = Some(now);
        } else if *ctx.local.last_b6_value {
            *ctx.local.last_b6_time = Some(now);
        }
        *ctx.local.last_b6_value = b6_high;

        let b7_high = ctx.local.b7.is_high().unwrap();
        if b7_high {
            next_input.numpad.seven = true;
            (
                next_input.keypad.p,
                next_input.keypad.q,
                next_input.keypad.r,
                next_input.keypad.s,
            ) = check_four_input(now, *ctx.local.last_b7_time, ctx.local.last_b7_click);
            *ctx.local.last_b7_time = Some(now);
        } else if *ctx.local.last_b7_value {
            *ctx.local.last_b7_time = Some(now);
        }
        *ctx.local.last_b7_value = b7_high;

        let b8_high = ctx.local.b8.is_high().unwrap();
        if b8_high {
            next_input.numpad.eight = true;
            (
                next_input.keypad.t,
                next_input.keypad.u,
                next_input.keypad.v,
            ) = check_three_input(now, *ctx.local.last_b8_time, ctx.local.last_b8_click);
            *ctx.local.last_b8_time = Some(now);
        } else if *ctx.local.last_b8_value {
            *ctx.local.last_b8_time = Some(now);
        }
        *ctx.local.last_b8_value = b8_high;

        let b9_high = ctx.local.b9.is_high().unwrap();
        if b9_high {
            next_input.numpad.nine = true;
            (
                next_input.keypad.w,
                next_input.keypad.x,
                next_input.keypad.y,
                next_input.keypad.z
            ) = check_four_input(now, *ctx.local.last_b9_time, ctx.local.last_b9_click);
            *ctx.local.last_b9_time = Some(now);
        } else {
            *ctx.local.last_b9_time = Some(now);
        }
        *ctx.local.last_b9_value = b9_high;

        if ctx.local.bback.is_high().unwrap() {
            next_input.keypad.backspace = true;
        }

        if ctx.local.b0.is_high().unwrap() {
            next_input.numpad.zero = true;
        }

        if ctx.local.bfront.is_high().unwrap() {
            next_input.keypad.enter = true;
        }

        ctx.shared.input_state.lock(|input_state| {
            *input_state = next_input;
        })
    }

    #[task(
        shared = [
            program_i2c,
            input_state,
            resets,
            ext1_decode_instructions,
            ext2_decode_instructions,
        ],
        priority = 2,
        binds = I2C1_IRQ
    )]
    /// Interrupt called when the program makes an I2C Request to the input controller
    fn i2c_interrupt(mut ctx: i2c_interrupt::Context) {
        (
            ctx.shared.program_i2c,
            ctx.shared.input_state,
            ctx.shared.resets,
        ).lock(|program_i2c, input, resets| {
            let mut instruction = None;
            loop {
                let mut i2c = program_i2c.take().unwrap();
                let event = i2c.next();
                if event.is_none() {
                    break;
                }

                let i2c = match event.unwrap() {
                    0 | 1 => {
                        // Start or Restart
                        let mut buffer = [0u8];
                        i2c.read(&mut buffer);
                        instruction = Some(InputRequest::from(buffer[0]));
                        i2c
                    },
                    2 => {
                        // Transfer Read
                        if let Some(instruction) = instruction {
                            match instruction {
                                InputRequest::FullInput => {
                                    let mut buffer = [0u8; 71];
                                    input.pack(&mut buffer).unwrap();
                                    i2c.write(&buffer);
                                },
                                InputRequest::Numpad => {
                                    let mut buffer = [0u8; 2];
                                    input.numpad.pack(&mut buffer).unwrap();
                                    i2c.write(&buffer);
                                },
                                InputRequest::Keypad => {
                                    let mut buffer = [0u8; 4];
                                    input.keypad.pack(&mut buffer).unwrap();
                                    i2c.write(&buffer);
                                },
                                InputRequest::Auxiliary => {
                                    let mut buffer = [0u8; 4];
                                    input.auxiliary.pack(&mut buffer).unwrap();
                                    i2c.write(&buffer);
                                },
                                InputRequest::Analog => {
                                    let mut buffer = [0u8; 12];
                                    input.analog.pack(&mut buffer).unwrap();
                                    i2c.write(&buffer);
                                },
                                InputRequest::DecodeOne => {
                                    ctx.shared.ext1_decode_instructions.lock(|decode_instructions| {
                                        i2c.write(decode_instructions);
                                    });
                                },
                                InputRequest::OtherOne => {
                                    i2c.write(&input.other_input_one);
                                },
                                InputRequest::DecodeTwo => {
                                    ctx.shared.ext2_decode_instructions.lock(|decode_instructions| {
                                        i2c.write(decode_instructions);
                                    });
                                },
                                InputRequest::OtherTwo => {
                                    i2c.write(&input.other_input_two);
                                },
                                InputRequest::SetAddress => {
                                    i2c.write(&[unsafe { I2C_ADDRESS }]);
                                },
                            }
                        }
                        i2c
                    },
                    3 => {
                        // Transfer Write
                        if let Some(instruction) = instruction {
                            match instruction {
                                InputRequest::SetAddress => {
                                    let mut buffer = [0u8];
                                    i2c.read(&mut buffer);
                                    unsafe { I2C_ADDRESS = buffer[0] };
                                    let (block, pins) = i2c.free(resets);
                                    I2C::new_peripheral_event_iterator(block, pins.0, pins.1, resets, unsafe { I2C_ADDRESS } )
                                },
                                _ => i2c,
                            }
                        } else {
                            i2c
                        }
                    },
                    _ => {
                        // Stop
                        instruction = None;
                        i2c
                    }
                };
                *program_i2c = Some(i2c);
            }
        });
    }

    #[task(
        shared = [
            ext1_enabled,
            ext2_enabled,
            en_ext1,
            en_ext2,
            ext1_spi,
            ext2_spi,
            ext1_decode_instructions,
            ext2_decode_instructions,
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

        // Get decode instructions from extension 1
        if new_one {
            (
                ctx.shared.ext1_spi,
                ctx.shared.ext1_decode_instructions
            ).lock(|ext1_spi, decode_instructions| {
                let mut buffer = [0u8; 248];
                ext1_spi.write(&[InputRequest::DecodeOne as u8]).unwrap();
                ext1_spi.transfer_in_place(&mut buffer).unwrap();
                *decode_instructions = buffer;
            });
        }

        // Get decode instructions form extension 2
        if new_two {
            (
                ctx.shared.ext2_spi,
                ctx.shared.ext2_decode_instructions
            ).lock(|ext2_spi, decode_instructions| {
                let mut buffer = [0u8; 248];
                ext2_spi.write(&[InputRequest::DecodeOne as u8]).unwrap();
                ext2_spi.transfer_in_place(&mut buffer).unwrap();
                *decode_instructions = buffer;
            });
        }
    }
}
