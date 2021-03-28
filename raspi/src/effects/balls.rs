use educe::Educe;
use rand::Rng;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{colour::HSV, controller::Controller, db, effects::prelude::*};

struct Ball {
	pos:   f32,
	speed: f32,
	color: [u8; 3],
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Educe)]
#[educe(Default)]
pub struct BallsConfig {
	#[educe(Default = 0.6)]
	darken_factor: f32,
}

pub struct Balls {
	config: BallsConfig,
	db:     sled::Tree,

	init:  bool,
	balls: Vec<Vec<Ball>>,
}

impl Balls {
	pub fn new(db: sled::Tree) -> Self {
		let mut effect = Balls {
			config: db::load_effect_config(&db),
			db,

			init: false,
			balls: Vec::new(),
		};

		effect.set_config(effect.config);

		effect
	}

	fn set_config(&mut self, config: BallsConfig) {
		self.config = config;
	}

	fn run(&mut self, ctrl: &mut Controller) {
		let mut state = ctrl.views_mut();

		if !self.init {
			for strip in state.iter_mut() {
				let mut balls_for_strip = Vec::new();
				for _ in 0..strip.len() / 25 {
					balls_for_strip.push(Ball {
						pos:   0.0,
						speed: rand::thread_rng().gen_range(0.2, 0.8),
						color: HSV::new(rand::random(), 255, 255).into(),
					})
				}
				self.balls.push(balls_for_strip);
			}

			self.init = true;
		}

		for strip in state.iter_mut() {
			for led in strip.iter_mut() {
				*led = darken_rgb(*led, self.config.darken_factor);
			}
		}

		for (i, section) in state.iter_mut().enumerate() {
			let balls = self.balls.get_mut(i).unwrap();
			let len = section.len() as f32;

			for ball in balls {
				ball.pos += ball.speed;
				while ball.pos < 0.0 || ball.pos > len {
					// debug!("fixing: {} len: {}", ball.pos, len);
					if ball.pos < 0.0 {
						ball.pos = -ball.pos;
					} else if ball.pos > len {
						ball.pos = len - (ball.pos - len);
					}
					ball.speed = -ball.speed;
					if rand::thread_rng().gen_range(0.0, 1.0) >= 0.7 {
						ball.color = HSV::new(rand::random(), 255, 255).into();
					}
				}
				section[ball.pos.floor().min(len - 1.0).max(0.0) as usize] = ball.color;
			}
		}

		ctrl.write_state();
	}
}

effect!(Balls, BallsConfig);
