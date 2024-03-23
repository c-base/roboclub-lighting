use educe::Educe;
use rand::Rng;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::effects::{config::color::ColorGradient, prelude::*, EffectWindow};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct BallsConfig {
	// #[schema(inline)]
	colors: ColorGradient,

	#[schema(minimum = 0.00001, maximum = 0.99999)]
	#[educe(Default = 0.1)]
	darken_factor: f32,

	#[schema(minimum = 0.0, maximum = 100.0)]
	#[educe(Default = 0.8)]
	speed: f32,
}

struct Ball {
	pos:   f32,
	speed: f32,
	dir:   f32,
	color: Rgba,
}

#[derive(Default)]
pub struct BallsState {
	init:  bool,
	balls: Vec<Ball>,
}

pub fn balls(config: &BallsConfig, state: &mut BallsState, mut window: EffectWindow) {
	if !state.init {
		for _ in 0..window.len() / 25 {
			state.balls.push(Ball {
				pos:   0.0,
				speed: rand::thread_rng().gen_range(0.1..1.0),
				dir:   1.0,
				color: config.colors.random().into(),
			})
		}

		state.init = true;
	}

	for led in window.iter_mut() {
		*led = led.darken(config.darken_factor).into();
	}

	let before: Vec<Rgba> = window.iter().copied().collect();
	for led in window.iter_mut() {
		*led = Default::default();
	}

	let len = window.len() as f32;

	for ball in state.balls.iter_mut() {
		ball.pos += ball.speed * ball.dir * config.speed;
		while ball.pos < 0.0 || ball.pos > len {
			// debug!("fixing: {} len: {}", ball.pos, len);
			if ball.pos < 0.0 {
				ball.pos = -ball.pos;
			} else if ball.pos >= len {
				ball.pos = len - (ball.pos - len);
			}

			ball.dir = -ball.dir;
			if rand::thread_rng().gen_range(0.0..1.0) >= 0.7f32 {
				ball.color = config.colors.random().into();
				ball.speed = rand::thread_rng().gen_range(0.1..1.0);
			}
		}

		// section.set_aa_range([], ball.color);
		window.set_aa(ball.pos, &ball.color);
		// section[ball.pos.round().max(0.0).min(len - 1.0) as usize] = ball.color;
	}

	for (i, led) in window.iter_mut().enumerate() {
		let before = &before[i];
		*led = Rgb::new(
			led.red.max(before.red),
			led.green.max(before.green),
			led.blue.max(before.blue),
		)
		.into();
	}

	// ctrl.write_state();
}
