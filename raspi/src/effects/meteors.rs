use educe::Educe;
use rand::{thread_rng, Rng};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{controller::Controller, db, effects::prelude::*};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Educe)]
#[educe(Default)]
pub struct MeteorsConfig {
	#[educe(Default = 10)]
	meteor_size:        usize,
	#[educe(Default = 16)]
	meteor_trail_decay: u8,
	#[educe(Default = 40)]
	speed_delay:        u64,
	#[educe(Default = 4)]
	meteor_count:       usize,
}

pub struct Meteors {
	config: MeteorsConfig,
	db:     sled::Tree,

	offset:  u8,
	meteors: Vec<usize>,
}

impl Meteors {
	pub fn new(db: sled::Tree) -> Self {
		let mut meteors = Meteors {
			config: db::load_effect_config(&db),
			db,
			offset: 0,
			meteors: vec![],
		};

		meteors.set_config(meteors.config);

		meteors
	}

	fn set_config(&mut self, config: MeteorsConfig) {
		if self.meteors.len() != config.meteor_count {
			let leds_per_meteor = NUM_LEDS * 3 / config.meteor_count;
			let mut meteors = vec![0; config.meteor_count];
			for i in 0..config.meteor_count {
				meteors[i] = leds_per_meteor * i;
			}
			self.meteors = meteors;
		}

		self.config = config;
	}

	fn run(&mut self, ctrl: &mut Controller) {
		let mut rand = thread_rng();

		let leds = ctrl.state_mut_flat();

		// fade brightness all LEDs one step
		for j in 0..leds.len() {
			fade_to_black_col(
				&mut leds[j],
				rand.gen_range(0, self.config.meteor_trail_decay),
				rand.gen_range(0, self.config.meteor_trail_decay),
				rand.gen_range(0, self.config.meteor_trail_decay),
			)
		}

		for counter in self.meteors.iter_mut() {
			if *counter > leds.len() + self.config.meteor_size * 2 {
				*counter = 0;
				// for i in 0..NUM_LEDS {
				// 	leds[i] = RGB::default();
				// }
			}

			// draw meteor
			for j in 0..self.config.meteor_size.min(*counter) {
				if (*counter - j < leds.len()) && (*counter + 1 - j != 0) {
					leds[*counter - j] = RGB::from(
						HSV::new(
							((self.offset as usize + j + *counter) % 256) as u8,
							255,
							255,
						)
						.to_rgb(),
					)
					.into();
				}
			}

			*counter += 1;
		}
		self.offset.wrapping_add(2);
		ctrl.write_state();
		sleep_ms(self.config.speed_delay);
	}
}

effect!(Meteors, MeteorsConfig);
