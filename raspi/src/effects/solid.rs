use color_eyre::owo_colors::OwoColorize;
use educe::Educe;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{config::color::Color, db, effects::prelude::*};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Educe)]
#[educe(Default)]
pub struct SolidConfig {
	color: Color,
}

pub struct Solid {
	config: SolidConfig,
	db:     sled::Tree,
}

impl Solid {
	pub fn new(mut db: sled::Tree) -> Self {
		let mut effect = Solid {
			config: db::load_config(&mut db),
			db,
		};

		effect.set_config(effect.config);

		effect
	}

	fn set_config(&mut self, config: SolidConfig) {
		self.config = config;
	}

	fn run(&mut self, ctrl: &mut impl LedController) {
		for led in ctrl.state_mut_flat() {
			*led = self.config.color.value().into();
		}
		ctrl.write_state();
		sleep_ms(50);
	}
}

effect!(Solid, SolidConfig);
