use educe::Educe;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
	colour::HSV,
	controller::Controller,
	db,
	effects::{prelude, prelude::*, Effect},
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Educe)]
#[educe(Default)]
pub struct FlashRainbowConfig {
	#[educe(Default = 10)]
	delay_on:  u64,
	#[educe(Default = 200)]
	delay_off: u64,
}

pub struct FlashRainbow {
	config: FlashRainbowConfig,
	db:     sled::Tree,
}

impl FlashRainbow {
	pub fn new(db: sled::Tree) -> Self {
		let mut effect = FlashRainbow {
			config: db::load_effect_config(&db),
			db,
		};

		effect.set_config(effect.config);

		effect
	}

	fn set_config(&mut self, config: FlashRainbowConfig) {
		self.config = config;
	}

	fn run(&mut self, ctrl: &mut Controller) {
		let color = rand::random();
		set_all_delay(
			ctrl,
			HSV::new(color, 255, 255).into(),
			true,
			self.config.delay_on,
		);
		set_all_delay(ctrl, [0, 0, 0], false, self.config.delay_off);
	}
}

effect!(FlashRainbow, FlashRainbowConfig);
