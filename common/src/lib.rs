#![no_std]

pub mod messages {
	pub const NO_UPDATE: u8 = 0x0;
	pub const UPDATE_LEDS: u8 = 0x1;
	pub const APPLY_LEDS: u8 = 0x2;
}

pub const LEDS_PER_STRIP: usize = 480;
/// Number of strips in parallel, do not change, pins are hardcoded
pub const STRIPS: usize = 3;

pub const TRANSFER_BUFFER_SIZE: usize = 3 * LEDS_PER_STRIP * STRIPS;

pub type LEDs = [[[u8; 3]; LEDS_PER_STRIP]; STRIPS];
pub type LedTransferBuffer = [u8; TRANSFER_BUFFER_SIZE];
