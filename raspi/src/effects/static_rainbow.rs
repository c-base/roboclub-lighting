use educe::Educe;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{config::db, effects::prelude::*};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct StaticRainbowConfig {
	#[schema(minimum = 0.01, maximum = 1000.0)]
	#[educe(Default = 255.0)]
	hue_frequency: f32,
}

pub struct StaticRainbow {
	config: StaticRainbowConfig,
	db:     sled::Tree,
}

impl StaticRainbow {
	pub fn new(mut db: sled::Tree) -> Self {
		let mut effect = StaticRainbow {
			config: db::load_config(&mut db),
			db,
		};

		effect.set_config(effect.config);

		effect
	}

	fn set_config(&mut self, config: StaticRainbowConfig) {
		self.config = config;
	}

	fn run(&mut self, ctrl: &mut impl LedController) {
		let leds = ctrl.state_mut_flat();

		for i in 0..leds.len() {
			let hue = (i as f32 * (360.0 / self.config.hue_frequency)) % 360.0;
			leds[i] = Hsv::new(hue, 1.0, 1.0).into();
		}

		ctrl.write_state();
	}
}

effect!(StaticRainbow, StaticRainbowConfig);
