use std::collections::HashMap;

use educe::Educe;
use eyre::{bail, eyre, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, error};

use crate::{
	controller::{Controller, ControllerConfig},
	effects::{prelude::Timer, Effect, EffectData},
};

pub struct EffectRunner {
	db:         sled::Tree,
	effects:    HashMap<String, Box<dyn Effect>>,
	controller: Controller,

	active_effect: String,
	timer:         Timer,
	counter:       usize,
}

// #[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Default)]
// pub struct GlobalConfig {
// 	controller: ControllerConfig,
// 	effects: HashMap<String, EffectData>,
// }

pub trait EffectAPI {
	// fn get_config(&self) -> GlobalConfig;
	fn get_controller_config(&self) -> Result<ControllerConfig>;
	fn set_controller_config(&mut self, config: ControllerConfig) -> Result<ControllerConfig>;
	fn get_effects(&self) -> Result<HashMap<String, EffectData>>;
	fn get_active_effect(&self) -> Result<EffectData>;
	fn set_active_effect(&mut self, effect_name: String) -> Result<()>;
	fn set_effect_config(
		&mut self,
		effect_name: String,
		config: serde_json::Value,
	) -> Result<serde_json::Value>;
}

impl EffectRunner {
	pub fn new(
		db: sled::Tree,
		effects: HashMap<String, Box<dyn Effect>>,
		controller: Controller,
	) -> Self {
		let active_effect = db
			.get("active_effect")
			.expect("can't access db")
			.and_then(|v| String::from_utf8(v.to_vec()).ok())
			.filter(|name| effects.contains_key(name))
			.unwrap_or_else(|| {
				effects
					.keys()
					.next()
					.expect("should always have 1 effect in the effects map")
					.clone()
			});

		let mut runner = EffectRunner {
			db,
			effects,
			controller,

			active_effect: active_effect.clone(),
			timer: Timer::new(),
			counter: 0,
		};

		runner.set_active_effect(active_effect);

		runner
	}

	pub fn set_effect_config(
		&mut self,
		effect_name: String,
		config: serde_json::Value,
	) -> Result<EffectData> {
		let effect = self
			.effects
			.get_mut(&effect_name)
			.ok_or(eyre!("no effect with the name {}", effect_name))?;
		effect.set_config(config)?;
		Ok(EffectData {
			name:    effect_name,
			schema:  effect.schema(),
			config:  effect.config()?,
			presets: Default::default(),
		})
	}

	pub fn tick(&mut self) {
		let effect = self
			.effects
			.get_mut(&self.active_effect)
			.expect("should always have the active effect");

		effect.run(&mut self.controller);

		let stats = self.timer.tick();
		if self.counter == 0 {
			debug!(
				"avg time to update: {:.2}ms (now {:.2}ms, min {:.2}ms, max {:.2}ms)",
				stats.avg, stats.dt, stats.min, stats.max
			);
		}
		self.counter = (self.counter + 1) % 60;
	}
}

impl EffectAPI for EffectRunner {
	// fn get_config(&self) -> GlobalConfig;

	fn get_controller_config(&self) -> Result<ControllerConfig> {
		Ok(self.controller.get_config())
	}

	fn set_controller_config(&mut self, config: ControllerConfig) -> Result<ControllerConfig> {
		Ok(self.controller.set_config(config))
	}

	fn get_effects(&self) -> Result<HashMap<String, EffectData>> {
		let mut map = HashMap::with_capacity(self.effects.len());

		for (name, effect) in self.effects.iter() {
			map.insert(
				name.clone(),
				EffectData {
					name:    name.clone(),
					schema:  effect.schema(),
					config:  effect.config()?,
					presets: Default::default(),
				},
			);
		}

		Ok(map)
	}

	fn get_active_effect(&self) -> Result<EffectData> {
		let effect = self
			.effects
			.get(&self.active_effect)
			.expect("should only ever be set to a valid effect");

		Ok(EffectData {
			name:    self.active_effect.clone(),
			schema:  effect.schema(),
			config:  effect.config()?,
			presets: Default::default(),
		})
	}

	fn set_active_effect(&mut self, effect_name: String) -> Result<()> {
		if !(self.effects.contains_key(&effect_name)) {
			bail!("the effect `{}` doesn't exist", effect_name);
		}

		self.active_effect = effect_name;

		if let Err(err) = self
			.db
			.insert("active_effect", self.active_effect.as_bytes())
		{
			error!("failed to write active_effect to db: {}", err);
		}

		Ok(())
	}

	fn set_effect_config(&mut self, effect_name: String, config: Value) -> Result<Value> {
		let effect = match self.effects.get_mut(&effect_name) {
			Some(effect) => effect,
			None => {
				bail!("`{}` is not a valid effect", effect_name);
			}
		};

		effect.set_config(config)
	}
}
