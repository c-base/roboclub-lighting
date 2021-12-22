use std::{collections::HashMap, fmt::Debug};

use color_eyre::Result;
use schemars::{schema::RootSchema, schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EmptyConfig {}

pub trait Effect: Send + Sync {
	fn schema(&self) -> RootSchema {
		schema_for!(EmptyConfig)
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
	pub schema:  RootSchema,
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
			fn schema(&self) -> schemars::schema::RootSchema {
				schemars::schema_for!($config)
			}
			fn config(&self) -> color_eyre::Result<serde_json::Value> {
				Ok(serde_json::to_value(&self.config)?)
			}
			fn set_config(
				&mut self,
				value: serde_json::Value,
			) -> color_eyre::Result<serde_json::Value> {
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
