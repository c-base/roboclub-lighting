use std::{
	collections::VecDeque,
	time::{Duration, Instant},
};

use educe::Educe;
use palette::{IntoColor, Mix, Shade};
use rand::{thread_rng, Rng};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
	config::color::ColorGradient,
	controller::Controller,
	db,
	effects::{config::color::ColorConfig, prelude::*},
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Educe)]
#[educe(Default)]
pub struct ExplosionsConfig {
	colors: ColorGradient,

	#[educe(Default = 250)]
	explosion_interval: u64,
	#[educe(Default = 0.5)]
	start_speed:        f32,
	#[educe(Default = 0.99)]
	speed_falloff:      f32,
	#[educe(Default = 0.05)]
	darken_factor:      f32,
}

struct Explosion {
	strip: usize,
	pos:   i32,
	speed: f32,
	width: f32,
	col:   Hsv,
}

pub struct Explosions {
	config: ExplosionsConfig,
	db:     sled::Tree,

	last:       Instant,
	explosions: VecDeque<Explosion>,
}

impl Explosions {
	pub fn new(mut db: sled::Tree) -> Self {
		let mut effect = Explosions {
			config: db::load_config(&mut db),
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

	fn run(&mut self, ctrl: &mut impl LedController) {
		let mut duration = Duration::from_millis(self.config.explosion_interval);

		let mut rand = thread_rng();

		let mut state = ctrl.views_mut();
		// let mut counter = 0.0;
		// counter = (counter + 0.01) % 255.0;

		let now = Instant::now();
		if now - self.last > duration {
			let strip = rand.gen_range(0..state.len());
			let pos = rand.gen_range(0..state[strip].len() as i32);

			self.explosions.push_back(Explosion {
				strip,
				pos,
				speed: self.config.start_speed,
				width: 0.0,
				col: self.config.colors.random(),
			});
			self.last = now;
		}

		for strip in state.iter_mut() {
			for led in strip.iter_mut() {
				*led = led.darken(self.config.darken_factor).into();
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

				let col: Rgba = explosion.col.clone().into();

				// anti-aliasing™
				let lower_factor = 1.0 - lower_end % 1.0;
				let upper_factor = upper_end % 1.0;

				for i in 0..slice.len() {
					let cur = &slice[i];
					slice[i] = if i == 0 {
						cur.mix(&col, lower_factor).into()
					} else if i == slice.len() - 1 {
						cur.mix(&col, upper_factor).into()
					} else {
						explosion.col.clone().into()
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
