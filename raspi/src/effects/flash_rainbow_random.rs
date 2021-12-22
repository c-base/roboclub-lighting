use educe::Educe;
use rand::{prelude::*, random};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
	config::color::ColorGradient,
	controller::Controller,
	db,
	effects::{config::color::ColorConfig, prelude, prelude::*, Effect},
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Educe)]
#[educe(Default)]
pub struct FlashRainbowRandomConfig {
	colors: ColorGradient,

	#[educe(Default = 0.2)]
	period:        f32,
	#[educe(Default = 0.1)]
	on_percentage: f32,

	#[educe(Default = 2)]
	segments_on:  usize,
	#[educe(Default = true)]
	height_slice: bool,
	#[educe(Default = true)]
	avoid_last:   bool,
}

pub struct FlashRainbowRandom {
	config: FlashRainbowRandomConfig,
	db:     sled::Tree,

	on: Vec<usize>,
}

impl FlashRainbowRandom {
	pub fn new(mut db: sled::Tree) -> Self {
		let mut effect = FlashRainbowRandom {
			config: db::load_config(&mut db),
			db,

			on: Vec::new(),
		};

		effect.set_config(effect.config);

		effect
	}

	fn set_config(&mut self, config: FlashRainbowRandomConfig) {
		self.config = config;
	}

	fn run(&mut self, ctrl: &mut impl LedController) {
		let color = self.config.colors.random();
		let start = std::time::Instant::now();
		let mut views = ctrl.views_mut();

		let [mut s0, mut s1, mut s2, mut s3, mut s4, mut s5, mut s6, mut s7, mut s8, mut s9, mut s10, mut s11, mut s12, mut s13, s14] =
			views.sections;

		let mut slices = if self.config.height_slice {
			vec![
				vec![&mut s0, &mut s4],
				vec![&mut s1, &mut s5],
				vec![&mut s2, &mut s6, &mut s7],
				vec![&mut s3, &mut s12, &mut s13],
				vec![&mut s8, &mut s9, &mut s10],
				vec![&mut s11],
			]
		} else {
			vec![
				vec![&mut s0],
				vec![&mut s1],
				vec![&mut s2],
				vec![&mut s3],
				vec![&mut s4],
				vec![&mut s5],
				vec![&mut s6, &mut s7],
				vec![&mut s8, &mut s9, &mut s10],
				vec![&mut s11],
				vec![&mut s12, &mut s13],
			]
		};

		let mut options: Vec<_> = (0..slices.len())
			.filter(|v| {
				if self.config.avoid_last {
					!self.on.contains(v)
				} else {
					true
				}
			})
			.collect();

		self.on.clear();

		options.shuffle(&mut thread_rng());
		for set_on in options.into_iter().take(self.config.segments_on as usize) {
			for mut slices in slices.get_mut(set_on) {
				for slice in slices.iter_mut() {
					for led in slice.iter_mut() {
						*led = color.clone().into();
					}
				}
			}
			self.on.push(set_on);
		}

		ctrl.write_state();
		sleep_ms((self.config.period * self.config.on_percentage * 1000.0) as u64);
		set_all(ctrl, &Rgba::default());
		let now = std::time::Instant::now();
		let diff = now - start;
		sleep_ms((self.config.period * 1000.0) as u64 - diff.as_millis() as u64);
	}
}

effect!(FlashRainbowRandom, FlashRainbowRandomConfig);
