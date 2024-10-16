use std::{
	ops::Add,
	time::{Duration, Instant},
};

use educe::Educe;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::effects::{prelude::*, EffectWindow};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct MovingLightsConfig {
	#[schema(minimum = 1.0, maximum = 100.0)]
	#[educe(Default = 20.0)]
	frequency: f32,

	#[schema(minimum = 1.0, maximum = 100.0)]
	#[educe(Default = 15.0)]
	impulse_len: f32,

	#[schema(minimum = 0.001, maximum = 10.0)]
	#[educe(Default = 2.0)]
	pulse_delay_s: f32,
}

pub struct MovingLightsState {
	anim:            MovingLightStripsAnimation,
	next_light_time: Instant,
	timer:           TimerState,
}

impl Default for MovingLightsState {
	fn default() -> Self {
		MovingLightsState {
			anim:            MovingLightStripsAnimation::new(NUM_LEDS, 15),
			next_light_time: Instant::now(),
			timer:           TimerState::default(),
		}
	}
}

pub fn moving_lights(
	config: &MovingLightsConfig,
	state: &mut MovingLightsState,
	mut window: EffectWindow,
) {
	let frequency_s = Duration::from_secs_f32(1.0 / config.frequency);

	let now = Instant::now();
	if now >= state.next_light_time {
		state.anim.add_next_light_impulse();
		state.next_light_time = now.add(Duration::from_secs_f32(config.pulse_delay_s))
	}

	if state.timer.tick(frequency_s).triggered {
		state.anim.shift_all_pixels();
	}

	for (i, led) in window.iter_mut().enumerate() {
		*led = state.anim.as_slice()[i];
	}
}

// pub struct MovingLights {
// 	config: MovingLightsConfig,
// 	db:     sled::Tree,
//
// 	anim:            MovingLightStripsAnimation,
// 	next_light_time: Instant,
// }
//
// impl MovingLights {
// 	pub fn new(mut db: sled::Tree) -> Self {
// 		let mut effect = MovingLights {
// 			config: db::load_config(&mut db),
// 			db,
//
// 			anim: MovingLightStripsAnimation::new(NUM_LEDS, 15),
// 			next_light_time: Instant::now(),
// 		};
//
// 		effect.set_config(effect.config);
//
// 		effect
// 	}
//
// 	fn set_config(&mut self, config: MovingLightsConfig) {
// 		self.config = config;
// 	}
//
// 	fn run(&mut self, ctrl: &mut impl LedController) {
// 		let frequency_ms = 1000 / self.config.frequency;
//
// 		let now = Instant::now();
// 		if now >= self.next_light_time {
// 			self.anim.add_next_light_impulse();
// 			self.next_light_time = now.add(Duration::from_millis(self.config.pulse_delay_ms))
// 		}
// 		self.anim.shift_all_pixels();
//
// 		for strip in ctrl.state_mut() {
// 			let len = strip.len();
// 			strip.clone_from_slice(&self.anim.as_slice()[0..len]);
// 		}
//
// 		sleep_ms(frequency_ms);
// 	}
// }
//
// effect!(MovingLights, MovingLightsConfig);

pub struct MovingLightStripsAnimation {
	rgb_data:    Vec<Rgba>,
	impulse_len: usize,
}

impl MovingLightStripsAnimation {
	pub fn new(led_count: usize, impulse_len: usize) -> Self {
		MovingLightStripsAnimation {
			rgb_data: vec![Default::default(); led_count + impulse_len],
			impulse_len,
		}
	}

	pub fn as_slice(&self) -> &[Rgba] {
		&self.rgb_data[self.impulse_len..]
	}
}

impl MovingLightStripsAnimation {
	/// Shifts all pixel to the next position. Beginning is filled
	/// with black (0, 0, 0).
	fn shift_all_pixels(&mut self) {
		let upper_border = self.rgb_data.len();
		for i in 0..upper_border {
			// loop backwards
			let i = upper_border - 1 - i;

			if i == 0 {
				self.rgb_data[i] = Rgba::default();
			} else {
				self.rgb_data.swap(i, i - 1);
			}
		}
	}

	fn add_next_light_impulse(&mut self) {
		// let (r, g, b) = get_random_pixel_val();

		let i = rand::random::<f32>() * 360.0;
		let color = Hsv::new(i, 1.0, 1.0);

		for i in 0..self.impulse_len {
			let factor = 1.0 - ((i as f32 / (self.impulse_len as f32 / 2.0)) - 1.0).abs();
			let color: Hsv = color.darken(factor).into();
			self.rgb_data[i] = color.into();
		}

		// self.rgb_data[00] = darken_rgb(r, g, b, 0.1);
		// self.rgb_data[01] = darken_rgb(r, g, b, 0.2);
		// self.rgb_data[02] = darken_rgb(r, g, b, 0.4);
		// self.rgb_data[03] = darken_rgb(r, g, b, 0.6);
		// self.rgb_data[04] = darken_rgb(r, g, b, 0.7);
		// self.rgb_data[05] = darken_rgb(r, g, b, 0.8);
		// self.rgb_data[06] = darken_rgb(r, g, b, 0.9);
		// self.rgb_data[07] = [r, g, b];
		// self.rgb_data[08] = darken_rgb(r, g, b, 0.9);
		// self.rgb_data[09] = darken_rgb(r, g, b, 0.8);
		// self.rgb_data[10] = darken_rgb(r, g, b, 0.7);
		// self.rgb_data[11] = darken_rgb(r, g, b, 0.6);
		// self.rgb_data[12] = darken_rgb(r, g, b, 0.4);
		// self.rgb_data[13] = darken_rgb(r, g, b, 0.2);
		// self.rgb_data[14] = darken_rgb(r, g, b, 0.1);
	}
}
