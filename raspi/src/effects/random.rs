use educe::Educe;
use palette::Saturate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{config::color::Color, controller::Controller, db, effects::prelude::*, noise};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Educe, ToSchema)]
#[educe(Default)]
pub struct RandomNoiseConfig {
	#[schema(inline)]
	color: Color,

	#[educe(Default = 0.03)]
	speed: f32,
	#[educe(Default = 20.0)]
	size:  f32,
	// #[educe(Default = 1.0)]
	// brightness: f32,
}

pub struct RandomNoise {
	config: RandomNoiseConfig,
	db:     sled::Tree,

	counter: f32,
}

impl RandomNoise {
	pub fn new(mut db: sled::Tree) -> Self {
		let mut effect = RandomNoise {
			config: db::load_config(&mut db),
			db,

			counter: 0.0,
		};

		effect.set_config(effect.config);

		effect
	}

	fn set_config(&mut self, config: RandomNoiseConfig) {
		self.config = config;
	}

	fn run(&mut self, ctrl: &mut impl LedController) {
		let state = ctrl.state_mut();

		self.counter += self.config.speed;

		for strip in 0..state.len() {
			// for i in 0..state[strip].len() / 8 {
			// 	let col = HSV::new(rand.gen(), 255, 255).into();
			//
			// 	let slice = &mut state[strip][i * 8..i * 8 + 8];
			// 	for i in slice {
			// 		*i = col;
			// 	}
			// }
			// for i in 0..state[strip].len() {
			// 	let col = HSV::new(rand.gen(), 255, 255).into();
			//
			// 	state[strip][i] = col;
			// 	// for i in slice {
			// 	// 	*i = col;
			// 	// }
			// }
			for i in 0..state[strip].len() {
				// let num = rand.gen_range(0, 55);
				// let num = rand.gen_range(0.0, 1.0);
				let num = 0.0;
				let num = (noise::simplex3d(i as f32 / self.config.size, num as f32, self.counter));
				// let num = if num > 0 {
				// 	self.config.brightness / num
				// } else {
				// 	num
				// };
				// let num = (num as f32 * (self.config.brightness));

				let col: Hsv = self.config.color.value().darken(1.0 - num).into();

				state[strip][i] = col.into();
			}
		}

		ctrl.write_state();
		// sleep_ms(1000);
	}
}

effect!(RandomNoise, RandomNoiseConfig);
