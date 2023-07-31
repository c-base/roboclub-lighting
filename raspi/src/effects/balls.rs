use educe::Educe;
use palette::{IntoColor, Shade, WithAlpha};
use rand::Rng;
use serde::{Deserialize, Serialize};
use utoipa::{openapi::RefOr, ToSchema};

use crate::{
	config::color::ColorGradient,
	controller::Controller,
	db,
	effects::{config::color::ColorConfig, prelude::*, schema::Schema},
};

struct Ball {
	pos:   f32,
	speed: f32,
	dir:   f32,
	color: Rgba,
}

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

pub struct Balls {
	config: BallsConfig,
	db:     sled::Tree,

	init:  bool,
	balls: Vec<Vec<Ball>>,
}

impl Balls {
	pub fn new(mut db: sled::Tree) -> Self {
		let mut effect = Balls {
			config: db::load_config(&mut db),
			db,

			init: false,
			balls: Vec::new(),
		};

		effect.set_config(effect.config);

		effect
	}

	fn set_config(&mut self, config: BallsConfig) {
		self.init = false;
		self.config = config;
	}

	fn run(&mut self, ctrl: &mut impl LedController) {
		let mut state = ctrl.views_mut();

		if !self.init {
			for strip in state.iter_mut() {
				let mut balls_for_strip = Vec::new();
				for _ in 0..strip.len() / 25 {
					balls_for_strip.push(Ball {
						pos:   0.0,
						speed: rand::thread_rng().gen_range(0.1..1.0),
						dir:   1.0,
						color: self.config.colors.random().into(),
					})
				}
				self.balls.push(balls_for_strip);
			}

			self.init = true;
		}

		for strip in state.iter_mut() {
			for led in strip.iter_mut() {
				*led = led.darken(self.config.darken_factor).into();
			}
		}

		for (i, section) in state.iter_mut().enumerate() {
			let before: Vec<Rgba> = section.iter_mut().map(|v| v.clone()).collect();
			for led in section.iter_mut() {
				*led = Default::default();
			}

			let balls = self.balls.get_mut(i).unwrap();
			let len = section.len() as f32;

			for ball in balls {
				ball.pos += ball.speed * ball.dir * self.config.speed;
				while ball.pos < 0.0 || ball.pos > len {
					// debug!("fixing: {} len: {}", ball.pos, len);
					if ball.pos < 0.0 {
						ball.pos = -ball.pos;
					} else if ball.pos >= len {
						ball.pos = len - (ball.pos - len);
					}

					ball.dir = -ball.dir;
					if rand::thread_rng().gen_range(0.0..1.0) >= 0.7f32 {
						ball.color = self.config.colors.random().into();
						ball.speed = rand::thread_rng().gen_range(0.1..1.0);
					}
				}

				// section.set_aa_range([], ball.color);
				section.set_aa(ball.pos, &ball.color);
				// section[ball.pos.round().max(0.0).min(len - 1.0) as usize] = ball.color;
			}

			for (i, led) in section.iter_mut().enumerate() {
				let before = &before[i];
				*led = Rgb::new(
					led.red.max(before.red),
					led.green.max(before.green),
					led.blue.max(before.blue),
				)
				.into();
			}
		}

		ctrl.write_state();
	}
}

effect!(Balls, BallsConfig);
