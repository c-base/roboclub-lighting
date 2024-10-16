use std::{
	collections::VecDeque,
	time::{Duration, Instant},
};

use educe::Educe;
use palette::Mix;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::effects::{config::color::ColorGradient, prelude::*, EffectWindow};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct ExplosionsConfig {
	// #[schema(inline)]
	colors: ColorGradient,

	#[schema(minimum = 0.001, maximum = 10.0)]
	#[educe(Default = 0.250)]
	explosion_interval: f32,

	#[schema(minimum = 0.0, maximum = 10.0)]
	#[educe(Default = 0.5)]
	start_speed: f32,

	#[schema(minimum = 0.00001, maximum = 0.99999)]
	#[educe(Default = 0.99)]
	speed_falloff: f32,

	#[schema(minimum = 0.00001, maximum = 0.99999)]
	#[educe(Default = 0.05)]
	darken_factor: f32,
}

struct Explosion {
	// strip: usize,
	pos:   i32,
	speed: f32,
	width: f32,
	col:   Hsv,
}

#[derive(Default)]
pub struct ExplosionsState {
	last:       Option<Instant>,
	explosions: VecDeque<Explosion>,
}

pub fn explosions(config: &ExplosionsConfig, state: &mut ExplosionsState, mut strip: EffectWindow) {
	let duration = Duration::from_secs_f32(config.explosion_interval);

	let mut rand = thread_rng();

	let now = Instant::now();
	let last = state.last.unwrap_or(Instant::now());

	if now - last > duration {
		let pos = rand.gen_range(0..strip.len() as i32);

		state.explosions.push_back(Explosion {
			pos,
			speed: config.start_speed,
			width: 0.0,
			col: config.colors.random(),
		});

		state.last = Some(now);
	}

	for led in strip.iter_mut() {
		*led = led.darken(config.darken_factor).into();
	}

	let mut pop_count = 0;
	for explosion in state.explosions.iter_mut() {
		// let start_width = explosion.width;
		explosion.width += explosion.speed;
		let end_width = explosion.width;

		explosion.speed *= config.speed_falloff;
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

			let col: Rgba = explosion.col.into();

			// anti-aliasing™
			let lower_factor = 1.0 - lower_end % 1.0;
			let upper_factor = upper_end % 1.0;

			for i in 0..slice.len() {
				let cur = &slice[i];
				slice[i] = if i == 0 {
					cur.mix(*col, lower_factor).into()
				} else if i == slice.len() - 1 {
					cur.mix(*col, upper_factor).into()
				} else {
					explosion.col.into()
				};
			}
		}
	}

	for _ in 0..pop_count {
		state.explosions.pop_front();
	}
}
