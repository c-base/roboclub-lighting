use educe::Educe;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{controller::Controller, db, effects::prelude::*};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Educe)]
#[educe(Default)]
pub struct EffectConfig {
	#[educe(Default = 10)]
	something: usize,
}

pub struct Effect {
	config: EffectConfig,
	db:     sled::Tree,
}

impl Effect {
	pub fn new(db: sled::Tree) -> Self {
		let mut effect = Effect {
			config: db::load_effect_config(&db),
			db,
		};

		effect.set_config(effect.config);

		effect
	}

	fn set_config(&mut self, config: EffectConfig) {
		self.config = config;
	}

	fn run(&mut self, ctrl: &mut impl LedController) {}
}

effect!(Effect, EffectConfig);
