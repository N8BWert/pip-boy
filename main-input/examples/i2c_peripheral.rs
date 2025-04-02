//!
//! Test program to test the i2c peripheral implementation
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
    use rp_pico::{hal::{self, clocks::init_clocks_and_plls, Sio, Watchdog, I2C}, pac::RESETS, Pins};

    use rtic_monotonics::rp2040_timer_monotonic;

    use main_input::peripherals::*;

    rp2040_timer_monotonic!(Mono);

    /// The address of this device on the i2c line
    static mut I2C_ADDRESS: u8 = 0;

    #[shared]
    struct Shared {
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

        let program_i2c = I2C::new_peripheral_event_iterator(
            ctx.device.I2C1,
            pins.gpio6.reconfigure(),
            pins.gpio7.reconfigure(),
            &mut ctx.device.RESETS,
            0u8,
        );

        let input_state = Input {
            numpad: NumpadBuilder::default()
                .one(true)
                .three(true)
                .five(true)
                .seven(true)
                .nine(true)
                .build()
                .unwrap(),
            keypad: KeypadBuilder::default()
                .a(true)
                .b(true)
                .c(true)
                .d(true)
                .build()
                .unwrap(),
            auxiliary: AuxiliaryBuilder::default()
                .and(true)
                .at(true)
                .backslash(true)
                .build()
                .unwrap(),
            analog: AnalogInputsBuilder::default()
                .a0(125)
                .a5(125)
                .build()
                .unwrap(),
            other_input_one: [0u8; 24],
            other_input_two: [0u8; 24],
        };

        hal::pac::NVIC::unpend(hal::pac::Interrupt::I2C1_IRQ);
        unsafe {
            hal::pac::NVIC::unmask(hal::pac::Interrupt::I2C1_IRQ);
        }

        (
            Shared {
                program_i2c: Some(program_i2c),
                input_state,
                resets: ctx.device.RESETS,
                ext1_decode_instructions: [5u8; 248],
                ext2_decode_instructions: [5u8; 248],
            },
            Local {

            }
        )
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
}