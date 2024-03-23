use std::time::{Duration, Instant};

pub use palette::{Darken, IntoColor, Mix};

pub use crate::{
	color::*,
	controller::{Controller, LedController},
	effects::Effect,
};

pub const NUM_LEDS: usize = crate::controller::LEDS_PER_STRIP;

pub fn sleep_ms(ms: u64) {
	std::thread::sleep(Duration::from_millis(ms));
}

fn fade(val: f32, fade_value: f32) -> f32 {
	if val <= 10.0 {
		0.0
	} else {
		val - (val * fade_value / 256.0)
	}
}

pub fn fade_to_black_col(led: &mut Rgba, fade_value_r: f32, fade_value_g: f32, fade_value_b: f32) {
	led.red = fade(led.red, fade_value_r);
	led.green = fade(led.green, fade_value_g);
	led.blue = fade(led.blue, fade_value_b);
}

pub fn set_all_delay(ctrl: &mut impl LedController, color: &Rgba, on: bool, delay_ms: u64) {
	let black = Rgba::default();
	set_all(ctrl, if on { color } else { &black });
	sleep_ms(delay_ms);
}

pub fn set_all(ctrl: &mut impl LedController, color: &Rgba) {
	let data = ctrl.state_mut_flat();
	for i in data {
		*i = *color;
	}
	todo!()
	// ctrl.write_state();
}

// pub fn darken_rgb(rgb: [u8; 3], factor: f32) -> [u8; 3] {
// 	[
// 		((rgb[0] as f32) * factor) as u8,
// 		((rgb[1] as f32) * factor) as u8,
// 		((rgb[2] as f32) * factor) as u8,
// 	]
// }
//
// pub fn blend_rgb(from: [u8; 3], to: [u8; 3], factor: f32) -> [u8; 3] {
// 	let mut iter = (0..3).map(|i| lerp(from[i] as f32, to[i] as f32, factor) as u8);
// 	[
// 		iter.next().unwrap(),
// 		iter.next().unwrap(),
// 		iter.next().unwrap(),
// 	]
// }

pub fn lerp(from: f32, to: f32, factor: f32) -> f32 {
	from + factor * (to - from)
}

pub struct Timer {
	last:           Instant,
	moving_min_max: [u128; 240],
}

impl Default for Timer {
	fn default() -> Self {
		Self::new()
	}
}

pub struct Stats {
	pub dt:  f32,
	pub avg: f32,
	pub min: f32,
	pub max: f32,
}

impl Timer {
	pub fn new() -> Self {
		Timer {
			last:           Instant::now(),
			moving_min_max: [0; 240],
		}
	}

	pub fn tick(&mut self) -> Stats {
		let current = Instant::now();
		let diff = current - self.last;
		self.last = current;

		self.moving_min_max.rotate_right(1);
		self.moving_min_max[0] = diff.as_micros();

		let avg: u128 =
			self.moving_min_max.iter().sum::<u128>() / self.moving_min_max.len() as u128;

		Stats {
			dt:  diff.as_micros() as f32 / 1000.0,
			avg: avg as f32 / 1000.0,
			min: self
				.moving_min_max
				.iter()
				.fold(u128::MAX, |min, cur| min.min(*cur)) as f32
				/ 1000.0,
			max: self
				.moving_min_max
				.iter()
				.fold(u128::MIN, |max, cur| max.max(*cur)) as f32
				/ 1000.0,
		}
	}
}
