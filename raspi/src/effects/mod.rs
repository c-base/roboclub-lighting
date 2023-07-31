use std::{collections::HashMap, fmt::Debug};

use eyre::Result;
use serde::{Deserialize, Serialize};
use utoipa::{openapi::Schema, schema, ToSchema};

use crate::effects::prelude::*;
pub use crate::effects::{
	balls::Balls,
	explosions::Explosions,
	flash_rainbow::FlashRainbow,
	flash_rainbow_noise::FlashRainbowNoise,
	flash_rainbow_random::FlashRainbowRandom,
	meteors::Meteors,
	moving_lights::MovingLights,
	police::Police,
	rainbow::Rainbow,
	random::RandomNoise,
	snake::Snake,
	solid::Solid,
	static_rainbow::StaticRainbow,
};

pub mod balls;
pub mod config;
pub mod explosions;
pub mod flash_rainbow;
pub mod flash_rainbow_noise;
pub mod flash_rainbow_random;
pub mod meteors;
pub mod moving_lights;
pub mod police;
pub mod prelude;
pub mod rainbow;
pub mod random;
pub mod schema;
pub mod snake;
pub mod solid;
pub mod static_rainbow;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema)]
pub struct EmptyConfig {}

pub trait Effect: Send + Sync {
	fn schema(&self) -> Schema {
		Schema::default()
	}

	fn config(&self) -> Result<serde_json::Value> {
		serde_json::to_value(EmptyConfig {}).map_err(|e| e.into())
	}

	fn set_config(&mut self, config: serde_json::Value) -> Result<serde_json::Value> {
		Ok(config)
	}

	fn run(&mut self, ctrl: &mut Controller);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectData {
	pub name:    String,
	pub schema:  Schema,
	pub config:  serde_json::Value,
	pub presets: HashMap<String, serde_json::Value>,
}

impl<FN> Effect for FN
where
	FN: FnMut(&mut Controller) + Send + Sync,
{
	fn run(&mut self, ctrl: &mut Controller) {
		self(ctrl)
	}
}

#[macro_export]
macro_rules! effect {
	($struct:ident, $config:ident) => {
		impl Effect for $struct {
			fn schema(&self) -> utoipa::openapi::Schema {
				match <$config as utoipa::ToSchema>::schema().1 {
					utoipa::openapi::RefOr::Ref(_) => {
						panic!(
							"Effect::schema needs to return a schema, {}::schema() returned a ref",
							stringify!($config)
						)
					}
					utoipa::openapi::RefOr::T(s) => s,
				}
			}
			fn config(&self) -> eyre::Result<serde_json::Value> {
				Ok(serde_json::to_value(&self.config)?)
			}
			fn set_config(&mut self, value: serde_json::Value) -> eyre::Result<serde_json::Value> {
				let config: $config = serde_json::from_value(value)?;
				$struct::set_config(self, config);
				crate::db::save_config(&mut self.db, &self.config)?;
				Ok(serde_json::to_value(&self.config)?)
			}

			fn run(&mut self, ctrl: &mut Controller) {
				$struct::run(self, ctrl);
			}
		}
	};
}
