use std::{
	collections::VecDeque,
	time::{Duration, Instant},
};

use educe::Educe;
use rand::{thread_rng, Rng};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
	colour::HSV,
	controller::Controller,
	db,
	effects::{prelude, prelude::*},
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Educe)]
#[educe(Default)]
pub struct ExplosionsConfig {
	#[educe(Default = 250)]
	explosion_interval: u64,
	#[educe(Default = 0.5)]
	start_speed:        f32,
	#[educe(Default = 0.99)]
	speed_falloff:      f32,
	#[educe(Default = 0.99)]
	darken_factor:      f32,

	#[educe(Default = 150)]
	hue_min: u8,
	#[educe(Default = 200)]
	hue_max: u8,
}

struct Explosion {
	strip: usize,
	pos:   i32,
	speed: f32,
	width: f32,
	col:   HSV,
}

pub struct Explosions {
	config: ExplosionsConfig,
	db:     sled::Tree,

	last:       Instant,
	explosions: VecDeque<Explosion>,
}

impl Explosions {
	pub fn new(db: sled::Tree) -> Self {
		let mut effect = Explosions {
			config: db::load_effect_config(&db),
			db,

			last: Instant::now(),
			explosions: VecDeque::new(),
		};

		effect.set_config(effect.config);

		effect
	}

	fn set_config(&mut self, config: ExplosionsConfig) {
		self.config = config;
	}

	fn run(&mut self, ctrl: &mut Controller) {
		let mut duration = Duration::from_millis(self.config.explosion_interval);

		let mut rand = thread_rng();

		let mut state = ctrl.views_mut();
		// let mut counter = 0.0;
		// counter = (counter + 0.01) % 255.0;

		let now = Instant::now();
		if now - self.last > duration {
			let strip = rand.gen_range(0, state.len());
			let pos = rand.gen_range(0, state[strip].len() as i32);

			self.explosions.push_back(Explosion {
				strip,
				pos,
				speed: self.config.start_speed,
				width: 0.0,
				col: HSV::new(
					((rand.gen_range(0, self.config.hue_max - self.config.hue_min)
						+ self.config.hue_min) % 255) as u8, //((rand.gen_range(0, 50) + counter as u16) % 255) as u8,
					255,
					255,
				),
			});
			self.last = now;
		}

		for strip in state.iter_mut() {
			for led in strip.iter_mut() {
				*led = prelude::darken_rgb(*led, self.config.darken_factor);
			}
		}

		let mut pop_count = 0;
		for explosion in self.explosions.iter_mut() {
			let mut strip = &mut state[explosion.strip];

			let start_width = explosion.width;
			explosion.width += explosion.speed;
			let end_width = explosion.width;

			explosion.speed *= self.config.speed_falloff;
			if explosion.speed <= 0.05 {
				pop_count += 1;
			}

			let lower_end = explosion.pos as f32 - end_width;
			let upper_end = explosion.pos as f32 + end_width;

			let len = strip.len();

			let lower_idx = (lower_end.floor() as usize).max(0);
			let upper_idx = (upper_end.ceil() as usize).min(len);

			if upper_idx > 0 && lower_idx <= len {
				let slice = &mut strip.range(lower_idx..upper_idx);

				let col = explosion.col.into();

				// anti-aliasing™
				let lower_factor = 1.0 - lower_end % 1.0;
				let upper_factor = upper_end % 1.0;

				for i in 0..slice.len() {
					let cur = slice[i];
					slice[i] = if i == 0 {
						prelude::blend_rgb(cur, col, lower_factor)
					} else if i == slice.len() - 1 {
						prelude::blend_rgb(cur, col, upper_factor)
					} else {
						explosion.col.into()
					};
				}
			}
		}
		for i in 0..pop_count {
			self.explosions.pop_front();
		}

		ctrl.write_state();
		// sleep_ms(100);
	}
}

effect!(Explosions, ExplosionsConfig);
