use std::time::Duration;

use educe::Educe;
use rand::random;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::effects::{config::color::ColorGradient, prelude::*, EffectWindow};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct FlashRainbowRandomConfig {
	// #[schema(inline)]
	colors: ColorGradient,

	#[schema(minimum = 0.00001, maximum = 5.0)]
	#[educe(Default = 0.2)]
	period: f32,

	#[schema(minimum = 0.00001, maximum = 0.99999)]
	#[educe(Default = 0.1)]
	on_percentage: f32,

	// #[schema(minimum = 1, maximum = 20)]
	// #[educe(Default = 2)]
	// segments_on: usize,
	#[schema(minimum = 0.01, maximum = 1.0)]
	#[educe(Default = 0.3)]
	on_probability: f32,

	// #[educe(Default = true)]
	// height_slice: bool,
	#[educe(Default = true)]
	avoid_last: bool,
}

#[derive(Default)]
pub struct FlashRainbowRandomState {
	timer:  TimerState,
	was_on: bool,
}

pub fn flash_rainbow_random(
	config: &FlashRainbowRandomConfig,
	state: &mut FlashRainbowRandomState,
	mut window: EffectWindow,
) {
	let period = Duration::from_secs_f32(config.period);
	let t = state.timer.tick(period);

	if !t.triggered {
		if t.percentage > config.on_percentage {
			clear_all_raw(&mut window);
		}

		return;
	}

	let color = config.colors.random();
	if config.avoid_last && state.was_on {
		state.was_on = false;
		return;
	}

	let val: f32 = random();

	if val >= config.on_probability {
		state.was_on = false;
		return;
	}

	state.was_on = true;
	set_all_raw(&mut window, color.into());
}

// pub struct FlashRainbowRandom {
// 	config: FlashRainbowRandomConfig,
// 	db:     sled::Tree,
//
// 	on: Vec<usize>,
// }
//
// impl FlashRainbowRandom {
// 	fn run(&mut self, ctrl: &mut impl LedController) {
// 		let color = self.config.colors.random();
// 		let start = std::time::Instant::now();
// 		let views = ctrl.views_mut();
//
// 		let [mut s0, mut s1, mut s2, mut s3, mut s4, mut s5, mut s6, mut s7, mut s8, mut s9, mut s10, mut s11, mut s12, mut s13, mut s14] =
// 			views.sections;
//
// 		let mut slices = if self.config.height_slice {
// 			vec![
// 				vec![&mut s0, &mut s4],
// 				vec![&mut s1, &mut s5],
// 				vec![&mut s2, &mut s6, &mut s7],
// 				vec![&mut s3, &mut s12, &mut s13],
// 				vec![&mut s8, &mut s9, &mut s10],
// 				vec![&mut s11],
// 				vec![&mut s14],
// 			]
// 		} else {
// 			vec![
// 				vec![&mut s0],
// 				vec![&mut s1],
// 				vec![&mut s2],
// 				vec![&mut s3],
// 				vec![&mut s4],
// 				vec![&mut s5],
// 				vec![&mut s6, &mut s7],
// 				vec![&mut s8, &mut s9, &mut s10],
// 				vec![&mut s11],
// 				vec![&mut s12, &mut s13],
// 				vec![&mut s14],
// 			]
// 		};
//
// 		let mut options: Vec<_> = (0..slices.len())
// 			.filter(|v| {
// 				if self.config.avoid_last {
// 					!self.on.contains(v)
// 				} else {
// 					true
// 				}
// 			})
// 			.collect();
//
// 		self.on.clear();
//
// 		options.shuffle(&mut thread_rng());
// 		for set_on in options.into_iter().take(self.config.segments_on as usize) {
// 			if let Some(slices) = slices.get_mut(set_on) {
// 				for slice in slices.iter_mut() {
// 					for led in slice.iter_mut() {
// 						*led = color.clone().into();
// 					}
// 				}
// 			}
// 			self.on.push(set_on);
// 		}
//
// 		ctrl.write_state();
// 		sleep_ms((self.config.period * self.config.on_percentage * 1000.0) as u64);
// 		set_all(ctrl, &Rgba::default());
// 		let now = std::time::Instant::now();
// 		let diff = now - start;
// 		sleep_ms((self.config.period * 1000.0) as u64 - diff.as_millis() as u64);
// 	}
// }
