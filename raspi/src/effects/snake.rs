use std::f32::consts::PI;

use educe::Educe;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{colour::HSV, controller::Controller, db, effects::prelude::*};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Educe)]
#[educe(Default)]
pub struct SnakeConfig {
	#[educe(Default = 0.25)]
	wave_speed:     f32,
	#[educe(Default = 64.0)]
	wave_frequency: f32,
	#[educe(Default = 1.0)]
	wave_influence: f32,

	#[educe(Default = 0.5)]
	hue_speed:  f32,
	#[educe(Default = 1.0)]
	hue_factor: f32,

	#[educe(Default = 150)]
	hue_min: u8,
	#[educe(Default = 200)]
	hue_max: u8,
}

pub struct Snake {
	config: SnakeConfig,
	db:     sled::Tree,

	wave_offset: f32,
	hue_offset:  f32,
}

impl Snake {
	pub fn new(db: sled::Tree) -> Self {
		let mut effect = Snake {
			config: db::load_effect_config(&db),
			db,

			wave_offset: 0.0,
			hue_offset: 0.0,
		};

		effect.set_config(effect.config);

		effect
	}

	fn set_config(&mut self, config: SnakeConfig) {
		self.config = config;
	}

	fn run(&mut self, ctrl: &mut Controller) {
		let state = ctrl.state_mut();

		let SnakeConfig {
			wave_speed,
			wave_frequency,
			wave_influence,
			hue_speed,
			hue_factor,
			hue_min,
			hue_max,
		} = self.config;

		self.wave_offset += wave_speed;
		self.hue_offset += hue_speed;

		for i in 0..NUM_LEDS {
			let progress: f32 = ((self.wave_offset + NUM_LEDS as f32 - i as f32 - 1.0)
				% wave_frequency) as f32
				/ wave_frequency * 2.0
				* PI;

			let val_top = 255 - (wave_influence * ((progress.sin() + 1.0) * 0.5 * 255.0)) as u8;
			let val_bottom =
				255 - (wave_influence * (((progress + PI).sin() + 1.0) * 0.5 * 255.0)) as u8;

			let hue = ((hue_min as u16
				+ (((i as f32 + self.hue_offset) * hue_factor) % (hue_max - hue_min) as f32)
					as u16) % 255) as u8;

			state[0][state[0].len() - i - 1] = HSV::new(hue, 255, val_top).into();
			state[1][state[1].len() - i - 1] = HSV::new(hue, 255, val_bottom).into();
			// state[1][i] = HSV::new(hue, 255, val_bottom).into();
			state[2][state[2].len() - i - 1] = HSV::new(hue, 255, val_bottom).into();
		}
		ctrl.write_state();
	}
}

effect!(Snake, SnakeConfig);
