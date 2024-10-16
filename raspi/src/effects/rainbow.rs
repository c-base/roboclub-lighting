use educe::Educe;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::effects::{prelude::*, EffectWindow};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct RainbowConfig {
	#[schema(minimum = 0.00001, maximum = 100.0)]
	#[educe(Default = 0.25)]
	wave_speed: f32,

	#[schema(minimum = 0.00001, maximum = 200.0)]
	#[educe(Default = 64.0)]
	wave_frequency: f32,

	#[schema(minimum = 0.0, maximum = 1.0)]
	#[educe(Default = 1.0)]
	wave_influence: f32,

	#[schema(minimum = 0.00001, maximum = 100.0)]
	#[educe(Default = 0.5)]
	hue_speed: f32,

	#[schema(minimum = 0.0, maximum = 1.0)]
	#[educe(Default = 1.0)]
	hue_factor: f32,
}

#[derive(Default)]
pub struct RainbowState {
	wave_offset: f32,
	hue_offset:  f32,
}

pub fn rainbow(config: &RainbowConfig, state: &mut RainbowState, mut window: EffectWindow) {
	state.wave_offset += config.wave_speed;
	state.hue_offset += config.hue_speed;

	let len = window.len();
	for (i, led) in window.iter_mut().enumerate() {
		let progress = ((state.wave_offset + len as f32 - i as f32 - 1.0) % config.wave_frequency)
			/ config.wave_frequency
			* 2.0 * core::f32::consts::PI;
		let val = 1.0 - (config.wave_influence * ((progress.sin() + 1.0) * 0.5));

		// let val = perlin.perlin(i as f32 / 10.0, progress / 20.0);
		// if i == 0 {
		//   print(val);
		// }

		let hue = (i as f32 + state.hue_offset * config.hue_factor) % 360.0;

		*led = Hsv::new(hue, 1.0, val).into();
	}
}

// pub struct Rainbow {
// 	config: RainbowConfig,
// 	db:     sled::Tree,
//
// 	wave_offset: f32,
// 	hue_offset:  f32,
// }
//
// impl Rainbow {
// 	fn run(&mut self, ctrl: &mut impl LedController) {
// 		let leds = ctrl.state_mut_flat();
//
// 		self.wave_offset += self.config.wave_speed;
// 		self.hue_offset += self.config.hue_speed;
//
// 		for i in 0..leds.len() {
// 			let progress = ((self.wave_offset + leds.len() as f32 - i as f32 - 1.0)
// 				% self.config.wave_frequency)
// 				/ self.config.wave_frequency
// 				* 2.0 * core::f32::consts::PI;
// 			let val = 1.0 - (self.config.wave_influence * ((progress.sin() + 1.0) * 0.5));
//
// 			// let val = perlin.perlin(i as f32 / 10.0, progress / 20.0);
// 			// if i == 0 {
// 			//   print(val);
// 			// }
//
// 			let hue = (i as f32 + self.hue_offset * self.config.hue_factor) % 360.0;
//
// 			leds[i] = Hsv::new(hue, 1.0, val).into();
// 		}
//
// 		ctrl.write_state();
// 	}
// }
