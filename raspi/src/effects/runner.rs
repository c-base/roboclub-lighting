use std::collections::HashMap;

use tracing::{debug, error};

use crate::{
	controller::Controller,
	effects::{prelude::Timer, Effect, EffectData},
};

pub struct EffectRunner {
	db:            sled::Tree,
	effects:       HashMap<String, Box<dyn Effect>>,
	active_effect: String,
	timer:         Timer,
	counter:       usize,
}

impl EffectRunner {
	pub fn new(db: sled::Tree, effects: HashMap<String, Box<dyn Effect>>) -> Self {
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
			active_effect: active_effect.clone(),
			timer: Timer::new(),
			counter: 0,
		};

		runner.set_active_effect(active_effect);

		runner
	}

	// pub fn effect_names(&self) -> Vec<String> {
	// 	self.effects.keys().cloned().collect()
	// }

	pub fn effects(&self) -> anyhow::Result<Vec<EffectData>> {
		let mut effects = Vec::with_capacity(self.effects.len());
		for (name, effect) in self.effects.iter() {
			effects.push(EffectData {
				name:   name.clone(),
				schema: effect.schema(),
				config: effect.config()?,
			})
		}
		Ok(effects)
	}

	pub fn active_effect(&self) -> anyhow::Result<EffectData> {
		let effect = self
			.effects
			.get(&self.active_effect)
			.expect("should only ever be set to a valid effect");
		Ok(EffectData {
			name:   self.active_effect.clone(),
			schema: effect.schema(),
			config: effect.config()?,
		})
	}

	pub fn set_active_effect(&mut self, effect: String) {
		if self.effects.contains_key(&effect) {
			self.active_effect = effect;

			if let Err(err) = self
				.db
				.insert("active_effect", self.active_effect.as_bytes())
			{
				error!("failed to write active_effect to db: {}", err);
			}
		}
	}

	pub fn get_current_effect_config(&self) -> anyhow::Result<serde_json::Value> {
		let effect = self
			.effects
			.get(&self.active_effect)
			.expect("should only ever be set to a valid effect");

		effect.config()
	}

	pub fn set_current_effect_config(&mut self, config: serde_json::Value) -> anyhow::Result<()> {
		let effect = self
			.effects
			.get_mut(&self.active_effect)
			.expect("should only ever be set to a valid effect");

		effect.set_config(config)
	}

	pub fn set_effect_config(
		&mut self,
		effect_name: String,
		config: serde_json::Value,
	) -> anyhow::Result<EffectData> {
		let effect = self
			.effects
			.get_mut(&effect_name)
			.ok_or(anyhow::anyhow!("no effect with the name {}", effect_name))?;
		effect.set_config(config)?;
		Ok(EffectData {
			name:   effect_name,
			schema: effect.schema(),
			config: effect.config()?,
		})
	}

	pub fn tick(&mut self, ctrl: &mut Controller) {
		let effect = self
			.effects
			.get_mut(&self.active_effect)
			.expect("should always have the active effect");

		effect.run(ctrl);

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
