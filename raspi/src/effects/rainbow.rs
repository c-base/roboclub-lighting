use educe::Educe;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{color::HSV, controller::Controller, db, effects::prelude::*};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Educe)]
#[educe(Default)]
pub struct RainbowConfig {
	#[educe(Default = 0.25)]
	wave_speed:     f32,
	#[educe(Default = 64.0)]
	wave_frequency: f32,
	#[educe(Default = 1.0)]
	wave_influence: f32,
	#[educe(Default = 0.5)]
	hue_speed:      f32,
	#[educe(Default = 1.0)]
	hue_factor:     f32,
}

pub struct Rainbow {
	config: RainbowConfig,
	db:     sled::Tree,

	wave_offset: f32,
	hue_offset:  f32,
}

impl Rainbow {
	pub fn new(mut db: sled::Tree) -> Self {
		let mut effect = Rainbow {
			config: db::load_effect_config(&mut db),
			db,

			wave_offset: 0.0,
			hue_offset: 0.0,
		};

		effect.set_config(effect.config);

		effect
	}

	fn set_config(&mut self, config: RainbowConfig) {
		self.config = config;
	}

	fn run(&mut self, ctrl: &mut Controller) {
		let leds = ctrl.state_mut_flat();

		self.wave_offset += self.config.wave_speed;
		self.hue_offset += self.config.hue_speed;

		for i in 0..leds.len() {
			let progress = ((self.wave_offset + leds.len() as f32 - i as f32 - 1.0)
				% self.config.wave_frequency)
				/ self.config.wave_frequency
				* 2.0 * core::f32::consts::PI;
			let val =
				255 - (self.config.wave_influence * ((progress.sin() + 1.0) * 0.5 * 255.0)) as u8;

			// let val = perlin.perlin(i as f32 / 10.0, progress / 20.0);
			// if i == 0 {
			//   print(val);
			// }

			let hue = ((i as f32 + self.hue_offset * self.config.hue_factor) % 255.0) as u8;

			leds[i] = HSV::new(hue, 255, val).into();
		}

		ctrl.write_state();
	}
}

effect!(Rainbow, RainbowConfig);
