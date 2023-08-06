use educe::Educe;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
	config::db,
	effects::{config::color::ColorGradient, prelude::*},
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct FlashRainbowConfig {
	// #[schema(inline)]
	colors: ColorGradient,

	#[schema(minimum = 0.00001, maximum = 5.0)]
	#[educe(Default = 0.2)]
	period: f32,

	#[schema(minimum = 0.00001, maximum = 0.99999)]
	#[educe(Default = 0.1)]
	on_percentage: f32,
}

pub struct FlashRainbow {
	config: FlashRainbowConfig,
	db:     sled::Tree,
}

impl FlashRainbow {
	pub fn new(mut db: sled::Tree) -> Self {
		let mut effect = FlashRainbow {
			config: db::load_config(&mut db),
			db,
		};

		effect.set_config(effect.config);

		effect
	}

	fn set_config(&mut self, config: FlashRainbowConfig) {
		self.config = config;
	}

	fn run(&mut self, ctrl: &mut impl LedController) {
		let color = self.config.colors.random();
		let start = std::time::Instant::now();
		set_all(ctrl, &color.clone().into());
		sleep_ms((self.config.period * self.config.on_percentage * 1000.0) as u64);
		set_all(ctrl, &Rgba::default());
		let now = std::time::Instant::now();
		let diff = now - start;
		sleep_ms((self.config.period * 1000.0) as u64 - diff.as_millis() as u64);
	}
}

effect!(FlashRainbow, FlashRainbowConfig);
