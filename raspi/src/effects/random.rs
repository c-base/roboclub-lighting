use educe::Educe;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{controller::Controller, db, effects::prelude::*, noise};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Educe)]
#[educe(Default)]
pub struct RandomNoiseConfig {
	#[educe(Default = 0.03)]
	speed: f32,
	#[educe(Default = 20.0)]
	size:  f32,
}

pub struct RandomNoise {
	config: RandomNoiseConfig,
	db:     sled::Tree,

	counter: f32,
}

impl RandomNoise {
	pub fn new(mut db: sled::Tree) -> Self {
		let mut effect = RandomNoise {
			config: db::load_effect_config(&mut db),
			db,

			counter: 0.0,
		};

		effect.set_config(effect.config);

		effect
	}

	fn set_config(&mut self, config: RandomNoiseConfig) {
		self.config = config;
	}

	fn run(&mut self, ctrl: &mut Controller) {
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
				let num = (noise::simplex3d(i as f32 / self.config.size, num as f32, self.counter)
					* 255.0) as u8;
				let col = [num, num, num];

				state[strip][i] = col;
				// for i in slice {
				// 	*i = col;
				// }
			}
		}

		ctrl.write_state();
		// sleep_ms(1000);
	}
}

effect!(RandomNoise, RandomNoiseConfig);
