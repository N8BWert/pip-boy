//!
//! Keypad Inputs
//! 

use core::ops::{BitOr, BitOrAssign};

use derive_builder::Builder;
use defmt::Format;
use crate::packing::{Pack, PackingError, Unpack};

#[derive(Clone, Copy, Debug, Format, PartialEq, Eq, Default, Builder)]
#[builder(build_fn(error(validation_error = false)))]
/// keypad input (a..z + shift + enter + backspace)
pub struct Keypad {
    #[builder(default = "false")]
    /// The shift button
    pub shift: bool,

    #[builder(default = "false")]
    /// The enter button
    pub enter: bool,

    #[builder(default = "false")]
    /// The backspace button
    pub backspace: bool,

    #[builder(default = "false")]
    /// The a button
    pub a: bool,

    #[builder(default = "false")]
    /// The b button
    pub b: bool,

    #[builder(default = "false")]
    /// The c button
    pub c: bool,

    #[builder(default = "false")]
    /// The d button
    pub d: bool,

    #[builder(default = "false")]
    /// The e button
    pub e: bool,

    #[builder(default = "false")]
    /// The f button
    pub f: bool,

    #[builder(default = "false")]
    /// The g button
    pub g: bool,

    #[builder(default = "false")]
    /// The h button
    pub h: bool,

    #[builder(default = "false")]
    /// The i button
    pub i: bool,

    #[builder(default = "false")]
    /// The j button
    pub j: bool,

    #[builder(default = "false")]
    /// The k button
    pub k: bool,

    #[builder(default = "false")]
    /// The l button
    pub l: bool,

    #[builder(default = "false")]
    /// The m button
    pub m: bool,

    #[builder(default = "false")]
    /// The n button
    pub n: bool,

    #[builder(default = "false")]
    /// The o button
    pub o: bool,

    #[builder(default = "false")]
    /// The p button
    pub p: bool,

    #[builder(default = "false")]
    /// The q button
    pub q: bool,

    #[builder(default = "false")]
    /// The r button
    pub r: bool,

    #[builder(default = "false")]
    /// The s button
    pub s: bool,

    #[builder(default = "false")]
    /// The t button
    pub t: bool,

    #[builder(default = "false")]
    /// The u button
    pub u: bool,

    #[builder(default = "false")]
    /// The v button
    pub v: bool,

    #[builder(default = "false")]
    /// The w button
    pub w: bool,

    #[builder(default = "false")]
    /// The x button
    pub x: bool,

    #[builder(default = "false")]
    /// The y button
    pub y: bool,

    #[builder(default = "false")]
    /// The z button
    pub z: bool,
}

impl Pack for Keypad {
    fn pack(self, buffer: &mut [u8]) -> Result<(), PackingError> {
        if buffer.len() < 4 {
            return Err(PackingError::InvalidBufferSize);
        }

        buffer[0] = ((self.shift as u8) << 7)
            | ((self.a as u8) << 6)
            | ((self.b as u8) << 5)
            | ((self.c as u8) << 4)
            | ((self.d as u8) << 3)
            | ((self.e as u8) << 2)
            | ((self.f as u8) << 1)
            | self.g as u8;
        buffer[1] = ((self.h as u8) << 7)
            | ((self.i as u8) << 6)
            | ((self.j as u8) << 5)
            | ((self.k as u8) << 4)
            | ((self.l as u8) << 3)
            | ((self.m as u8) << 2)
            | ((self.n as u8) << 1)
            | self.o as u8;
        buffer[2] = ((self.p as u8) << 7)
            | ((self.q as u8) << 6)
            | ((self.r as u8) << 5)
            | ((self.s as u8) << 4)
            | ((self.t as u8) << 3)
            | ((self.u as u8) << 2)
            | ((self.v as u8) << 1)
            | self.w as u8;
        buffer[3] = ((self.x as u8) << 7)
            | ((self.y as u8) << 6)
            | ((self.z as u8) << 5)
            | ((self.enter as u8) << 4)
            | ((self.backspace as u8) << 3);
        Ok(())
    }
}

impl Unpack for Keypad {
    fn unpack(buffer: &[u8]) -> Result<Self, PackingError>
    where
        Self: Sized,
    {
        if buffer.len() < 4 {
            return Err(PackingError::InvalidBufferSize);
        }

        Ok(Self {
            shift: buffer[0] & (1 << 7) != 0,
            a: buffer[0] & (1 << 6) != 0,
            b: buffer[0] & (1 << 5) != 0,
            c: buffer[0] & (1 << 4) != 0,
            d: buffer[0] & (1 << 3) != 0,
            e: buffer[0] & (1 << 2) != 0,
            f: buffer[0] & (1 << 1) != 0,
            g: buffer[0] & 1 != 0,
            h: buffer[1] & (1 << 7) != 0,
            i: buffer[1] & (1 << 6) != 0,
            j: buffer[1] & (1 << 5) != 0,
            k: buffer[1] & (1 << 4) != 0,
            l: buffer[1] & (1 << 3) != 0,
            m: buffer[1] & (1 << 2) != 0,
            n: buffer[1] & (1 << 1) != 0,
            o: buffer[1] & 1 != 0,
            p: buffer[2] & (1 << 7) != 0,
            q: buffer[2] & (1 << 6) != 0,
            r: buffer[2] & (1 << 5) != 0,
            s: buffer[2] & (1 << 4) != 0,
            t: buffer[2] & (1 << 3) != 0,
            u: buffer[2] & (1 << 2) != 0,
            v: buffer[2] & (1 << 1) != 0,
            w: buffer[2] & 1 != 0,
            x: buffer[3] & (1 << 7) != 0,
            y: buffer[3] & (1 << 6) != 0,
            z: buffer[3] & (1 << 5) != 0,
            enter: buffer[3] & (1 << 4) != 0,
            backspace: buffer[3] & (1 << 3) != 0,
        })
    }
}

impl BitOr for Keypad {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            shift: self.shift || rhs.shift,
            a: self.a || rhs.a,
            b: self.a || rhs.b,
            c: self.a || rhs.c,
            d: self.a || rhs.d,
            e: self.a || rhs.e,
            f: self.a || rhs.f,
            g: self.a || rhs.g,
            h: self.a || rhs.h,
            i: self.a || rhs.i,
            j: self.a || rhs.j,
            k: self.a || rhs.k,
            l: self.a || rhs.l,
            m: self.a || rhs.m,
            n: self.a || rhs.n,
            o: self.a || rhs.o,
            p: self.a || rhs.p,
            q: self.a || rhs.q,
            r: self.a || rhs.r,
            s: self.a || rhs.s,
            t: self.t || rhs.t,
            u: self.u || rhs.u,
            v: self.v || rhs.v,
            w: self.w || rhs.w,
            x: self.x || rhs.x,
            y: self.y || rhs.y,
            z: self.z || rhs.z,
            enter: self.enter || rhs.enter,
            backspace: self.backspace || rhs.backspace,
        }
    }
}

impl BitOrAssign for Keypad {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_keypad() {
        let keypad = KeypadBuilder::create_empty()
            .a(true)
            .c(true)
            .e(true)
            .g(true)
            .i(true)
            .k(true)
            .m(true)
            .o(true)
            .q(true)
            .s(true)
            .u(true)
            .w(true)
            .y(true)
            .build()
            .unwrap();

        let mut buffer = [0u8; 4];
        keypad.pack(&mut buffer).unwrap();
        assert_eq!(buffer, [0b0101_0101, 0b0101_0101, 0b0101_0101, 0b0100_0000]);
    }

    #[test]
    fn test_unpack_keypad() {
        let buffer = [0b1010_1010, 0b1010_1010, 0b1010_1010, 0b1010_0000];

        let keypad = Keypad::unpack(&buffer).unwrap();
        assert_eq!(
            keypad,
            KeypadBuilder::create_empty()
                .shift(true)
                .b(true)
                .d(true)
                .f(true)
                .h(true)
                .j(true)
                .l(true)
                .n(true)
                .p(true)
                .r(true)
                .t(true)
                .v(true)
                .x(true)
                .z(true)
                .build()
                .unwrap()
        );
    }

    #[test]
    fn test_pack_unpack_keypad() {
        let keypad = KeypadBuilder::create_empty()
            .a(true)
            .c(true)
            .e(true)
            .g(true)
            .i(true)
            .k(true)
            .m(true)
            .o(true)
            .p(true)
            .r(true)
            .t(true)
            .v(true)
            .x(true)
            .z(true)
            .build()
            .unwrap();

        let mut buffer = [0u8; 4];
        keypad.clone().pack(&mut buffer).unwrap();
        assert_eq!(keypad, Keypad::unpack(&buffer).unwrap(),);
    }

    #[test]
    fn test_bitor_keybad() {
        let keypad1 = KeypadBuilder::create_empty()
            .a(true)
            .c(true)
            .d(true)
            .build()
            .unwrap();
        let keypad2 = KeypadBuilder::create_empty()
            .a(true)
            .f(true)
            .z(true)
            .build()
            .unwrap();

        let keypad = keypad1 | keypad2;
        assert!(keypad.a);
        assert!(keypad.c);
        assert!(keypad.d);
        assert!(keypad.f);
        assert!(keypad.z);
    }
}