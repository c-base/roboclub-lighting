use std::f32::consts::PI;

use educe::Educe;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::effects::{config::color::ColorGradient, prelude::*, EffectWindow};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct SnakeConfig {
	// #[schema(inline)]
	colors: ColorGradient,

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
	#[educe(Default = 0.05)]
	hue_speed: f32,

	#[schema(minimum = 0.0, maximum = 1.0)]
	#[educe(Default = 0.1)]
	hue_factor: f32,
}

#[derive(Default)]
pub struct SnakeState {
	wave_offset: f32,
	hue_offset:  f32,
}

pub fn snake(config: &SnakeConfig, state: &mut SnakeState, mut window: EffectWindow) {
	// let state = ctrl.state_mut();

	let SnakeConfig {
		colors,
		wave_speed,
		wave_frequency,
		wave_influence,
		hue_speed,
		hue_factor,
	} = config;

	state.wave_offset += wave_speed;
	state.hue_offset += hue_speed;

	let leds = window.len();

	for i in 0..leds {
		let progress: f32 = ((state.wave_offset + leds as f32 - i as f32 - 1.0) % wave_frequency)
			/ wave_frequency
			* 2.0 * PI;

		let val_top = 1.0 - (wave_influence * ((progress.sin() + 1.0) * 0.5));
		// let val_bottom = 1.0 - (wave_influence * (((progress + PI).sin() + 1.0) * 0.5));

		let hue = ((i as f32 + state.hue_offset) * hue_factor) % 1.0;

		window[leds - i - 1] = colors.lerp(hue).darken(1.0 - val_top).into();

		// state[0][state[0].len() - i - 1] = colors.lerp(hue).darken(1.0 - val_top).into();
		// state[1][state[1].len() - i - 1] = colors.lerp(hue).darken(1.0 - val_bottom).into();
		// state[2][state[2].len() - i - 1] = colors.lerp(hue).darken(1.0 - val_bottom).into();

		// state[0][state[0].len() - i - 1] = Hsv::new(hue, 1.0, val_top).into();
		// state[1][state[1].len() - i - 1] = Hsv::new(hue, 1.0, val_bottom).into();
		// state[1][i] = HSV::new(hue, 255, val_bottom).into();
		// state[2][state[2].len() - i - 1] = Hsv::new(hue, 1.0, val_bottom).into();
	}
}

//
// pub struct Snake {
// 	config: SnakeConfig,
// 	db:     sled::Tree,
//
// 	wave_offset: f32,
// 	hue_offset:  f32,
// }
//
// impl Snake {
// 	fn run(&mut self, ctrl: &mut impl LedController) {
// 		let state = ctrl.state_mut();
//
// 		let SnakeConfig {
// 			colors,
// 			wave_speed,
// 			wave_frequency,
// 			wave_influence,
// 			hue_speed,
// 			hue_factor,
// 		} = self.config;
//
// 		self.wave_offset += wave_speed;
// 		self.hue_offset += hue_speed;
//
// 		for i in 0..NUM_LEDS {
// 			let progress: f32 = ((self.wave_offset + NUM_LEDS as f32 - i as f32 - 1.0)
// 				% wave_frequency) as f32
// 				/ wave_frequency * 2.0
// 				* PI;
//
// 			let val_top = 1.0 - (wave_influence * ((progress.sin() + 1.0) * 0.5));
// 			let val_bottom = 1.0 - (wave_influence * (((progress + PI).sin() + 1.0) * 0.5));
//
// 			let hue = ((i as f32 + self.hue_offset) * hue_factor) % 1.0;
//
// 			state[0][state[0].len() - i - 1] = colors.lerp(hue).darken(1.0 - val_top).into();
// 			state[1][state[1].len() - i - 1] = colors.lerp(hue).darken(1.0 - val_bottom).into();
// 			state[2][state[2].len() - i - 1] = colors.lerp(hue).darken(1.0 - val_bottom).into();
//
// 			// state[0][state[0].len() - i - 1] = Hsv::new(hue, 1.0, val_top).into();
// 			// state[1][state[1].len() - i - 1] = Hsv::new(hue, 1.0, val_bottom).into();
// 			// state[1][i] = HSV::new(hue, 255, val_bottom).into();
// 			// state[2][state[2].len() - i - 1] = Hsv::new(hue, 1.0, val_bottom).into();
// 		}
// 		ctrl.write_state();
// 	}
// }
