#![no_std]

pub mod hilbert;

use bytemuck::{CheckedBitPattern, NoUninit};


#[repr(u8)]
#[derive(Copy, Clone, Default, NoUninit, Debug)]
pub enum Terrian {
    #[default]
    Space = 0,
    SwallowWater = 1,
    DeepWater = 2,
}

unsafe impl CheckedBitPattern for Terrian {
    type Bits = u8;

    fn is_valid_bit_pattern(bits: &u8) -> bool {
        match *bits {
            0 | 1 | 2 => true,
            _ => false,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, NoUninit, CheckedBitPattern, Debug)]
pub struct Tile {
    pub terrian: Terrian,
    pub temperature: u8,
    pub pressure: u8,
    pub gravity: [i8; 3],
    pub pad: [u8; 2],
}

#[derive(Copy, Clone, NoUninit)]
#[repr(C)]
pub struct TileConstants {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub frame_number: u32,
}


