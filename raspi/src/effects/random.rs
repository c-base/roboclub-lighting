use educe::Educe;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
	effects::{config::color::Color, prelude::*, EffectWindow},
	noise,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct RandomNoiseConfig {
	// #[schema(inline)]
	color: Color,

	#[schema(minimum = 0.00001, maximum = 20.0)]
	#[educe(Default = 0.03)]
	speed: f32,

	#[schema(minimum = 0.00001, maximum = 500.0)]
	#[educe(Default = 20.0)]
	size: f32,
	// #[educe(Default = 1.0)]
	// brightness: f32,
}

#[derive(Default)]
pub struct RandomNoiseState {
	counter: f32,
}

pub fn random(config: &RandomNoiseConfig, state: &mut RandomNoiseState, mut window: EffectWindow) {
	state.counter += config.speed;

	for (i, led) in window.iter_mut().enumerate() {
		let num = 0.0;
		let num = noise::simplex3d(i as f32 / config.size, num as f32, state.counter);

		let col: Hsv = config.color.value().darken(1.0 - num).into();

		*led = col.into();
	}
}

// pub struct RandomNoise {
// 	config: RandomNoiseConfig,
// 	db:     sled::Tree,
//
// 	counter: f32,
// }
//
// impl RandomNoise {
// 	fn run(&mut self, ctrl: &mut impl LedController) {
// 		let state = ctrl.state_mut();
//
// 		self.counter += self.config.speed;
//
// 		for strip in 0..state.len() {
// 			// for i in 0..state[strip].len() / 8 {
// 			// 	let col = HSV::new(rand.gen(), 255, 255).into();
// 			//
// 			// 	let slice = &mut state[strip][i * 8..i * 8 + 8];
// 			// 	for i in slice {
// 			// 		*i = col;
// 			// 	}
// 			// }
// 			// for i in 0..state[strip].len() {
// 			// 	let col = HSV::new(rand.gen(), 255, 255).into();
// 			//
// 			// 	state[strip][i] = col;
// 			// 	// for i in slice {
// 			// 	// 	*i = col;
// 			// 	// }
// 			// }
// 			for i in 0..state[strip].len() {
// 				// let num = rand.gen_range(0, 55);
// 				// let num = rand.gen_range(0.0, 1.0);
// 				let num = 0.0;
// 				let num = noise::simplex3d(i as f32 / self.config.size, num as f32, self.counter);
// 				// let num = if num > 0 {
// 				// 	self.config.brightness / num
// 				// } else {
// 				// 	num
// 				// };
// 				// let num = (num as f32 * (self.config.brightness));
//
// 				let col: Hsv = self.config.color.value().darken(1.0 - num).into();
//
// 				state[strip][i] = col.into();
// 			}
// 		}
//
// 		ctrl.write_state();
// 		// sleep_ms(1000);
// 	}
// }
