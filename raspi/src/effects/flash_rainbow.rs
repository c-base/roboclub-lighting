use educe::Educe;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
	color::HSV,
	controller::Controller,
	db,
	effects::{config::color::ColorConfig, prelude, prelude::*, Effect},
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Educe)]
#[educe(Default)]
pub struct FlashRainbowConfig {
	#[educe(Default = 0.2)]
	period:        f32,
	#[educe(Default = 0.1)]
	on_percentage: f32,

	color: ColorConfig,
}

pub struct FlashRainbow {
	config: FlashRainbowConfig,
	db:     sled::Tree,
}

impl FlashRainbow {
	pub fn new(mut db: sled::Tree) -> Self {
		let mut effect = FlashRainbow {
			config: db::load_effect_config(&mut db),
			db,
		};

		effect.set_config(effect.config);

		effect
	}

	fn set_config(&mut self, config: FlashRainbowConfig) {
		self.config = config;
	}

	fn run(&mut self, ctrl: &mut Controller) {
		let color = self.config.color.random();
		let start = std::time::Instant::now();
		set_all(ctrl, color.into());
		sleep_ms((self.config.period * self.config.on_percentage * 1000.0) as u64);
		set_all(ctrl, [0, 0, 0]);
		let now = std::time::Instant::now();
		let diff = now - start;
		sleep_ms((self.config.period * 1000.0) as u64 - diff.as_millis() as u64);
	}
}

effect!(FlashRainbow, FlashRainbowConfig);
