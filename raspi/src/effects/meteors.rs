use educe::Educe;
use palette::IntoColor;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{controller::Controller, db, effects::prelude::*};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct MeteorsConfig {
	#[schema(minimum = 1, maximum = 100)]
	#[educe(Default = 10)]
	meteor_size: usize,

	#[schema(minimum = 0.00001, maximum = 100.0)]
	#[educe(Default = 16.0)]
	meteor_trail_decay: f32,

	#[schema(minimum = 1, maximum = 100)]
	#[educe(Default = 40)]
	speed_delay: u64,

	#[schema(minimum = 1, maximum = 100)]
	#[educe(Default = 4)]
	meteor_count: usize,
}

pub struct Meteors {
	config: MeteorsConfig,
	db:     sled::Tree,

	offset:  u8,
	meteors: Vec<usize>,
}

impl Meteors {
	pub fn new(mut db: sled::Tree) -> Self {
		let mut meteors = Meteors {
			config: db::load_config(&mut db),
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

	fn run(&mut self, ctrl: &mut impl LedController) {
		let mut rand = thread_rng();

		let leds = ctrl.state_mut_flat();

		// fade brightness all LEDs one step
		for j in 0..leds.len() {
			fade_to_black_col(
				&mut leds[j],
				rand.gen_range(0.0..self.config.meteor_trail_decay),
				rand.gen_range(0.0..self.config.meteor_trail_decay),
				rand.gen_range(0.0..self.config.meteor_trail_decay),
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
					leds[*counter - j] = Hsv::new(
						((self.offset as f32 + j as f32 + *counter as f32) % 360.0),
						1.0,
						1.0,
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
