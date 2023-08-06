use educe::Educe;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
	config::db,
	effects::{config::color::ColorGradient, prelude::*},
	noise,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct FlashRainbowNoiseConfig {
	// #[schema(inline)]
	colors: ColorGradient,

	#[schema(minimum = 0.00001, maximum = 5.0)]
	#[educe(Default = 0.15)]
	period: f32,

	#[schema(minimum = 0.00001, maximum = 0.99999)]
	#[educe(Default = 0.001)]
	on_percentage: f32,

	#[schema(minimum = 0.00001, maximum = 20.0)]
	#[educe(Default = 0.03)]
	speed: f32,

	#[schema(minimum = 0.00001, maximum = 500.0)]
	#[educe(Default = 20.0)]
	size: f32,

	#[schema(minimum = 0.0, maximum = 1.0)]
	#[educe(Default = 0.05)]
	threshold: f32,
}

pub struct FlashRainbowNoise {
	config: FlashRainbowNoiseConfig,
	db:     sled::Tree,

	counter: f32,
}

impl FlashRainbowNoise {
	pub fn new(mut db: sled::Tree) -> Self {
		let mut effect = FlashRainbowNoise {
			config: db::load_config(&mut db),
			db,

			counter: 0.0,
		};

		effect.set_config(effect.config);

		effect
	}

	fn set_config(&mut self, config: FlashRainbowNoiseConfig) {
		self.config = config;
	}

	fn run(&mut self, ctrl: &mut impl LedController) {
		let start = std::time::Instant::now();

		self.counter += self.config.speed;

		let color = self.config.colors.random();

		let data = ctrl.state_mut();
		for (strip_num, strip) in data.iter_mut().enumerate() {
			for (led_num, led) in strip.iter_mut().enumerate() {
				let noise_val = noise::simplex3d(
					led_num as f32 / self.config.size,
					strip_num as f32 * 100.0,
					self.counter,
				);
				if noise_val > self.config.threshold {
					*led = color.clone().into();
				}
			}
		}

		ctrl.write_state();
		sleep_ms((self.config.period * self.config.on_percentage * 1000.0) as u64);

		set_all(ctrl, &Rgba::default());
		let now = std::time::Instant::now();
		let diff = now - start;
		sleep_ms((self.config.period * 1000.0) as u64 - diff.as_millis() as u64);
	}
}

effect!(FlashRainbowNoise, FlashRainbowNoiseConfig);
