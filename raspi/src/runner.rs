use std::{collections::HashMap, path::Path};

use eyre::{bail, Result};
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tracing::debug;

use crate::{
	config::{Config, DisplayState, DisplayStateEffect, GlobalConfig, Presets},
	controller::{Controller, LedController},
	effects::{prelude::Timer, Effect, EffectData, EffectFactory},
};

type EffectsMap = HashMap<String, Box<dyn EffectFactory>>;

pub struct EffectRunner {
	effects:       EffectsMap,
	effects_state: Vec<Vec<(String, Box<dyn Effect>)>>,
	controller:    Controller,

	config:  Config<GlobalConfig>,
	state:   Config<DisplayState>,
	presets: Config<Presets>,

	state_notifier: Sender<DisplayState>,
	timer:          Timer,
	counter:        usize,
}

pub trait EffectAPI {
	fn get_global_config(&self) -> Result<&GlobalConfig>;
	fn set_global_config(&mut self, config: GlobalConfig) -> Result<()>;

	fn list_effects(&self) -> Result<HashMap<String, EffectData>>;

	fn list_presets(&self) -> Result<&HashMap<String, DisplayState>>;
	fn set_preset(&mut self, name: String, preset: DisplayState) -> Result<()>;
	fn delete_preset(&mut self, name: String) -> Result<()>;
	fn load_preset(&mut self, name: String) -> Result<()>;
	fn save_preset(&mut self, name: String) -> Result<()>;

	fn get_state(&self) -> Result<&DisplayState>;
	fn set_state(&mut self, state: DisplayState) -> Result<()>;

	fn subscribe(&self) -> Receiver<DisplayState>;
}

fn default_effect(effects: &EffectsMap) -> DisplayStateEffect {
	let default_effect = effects
		.keys()
		.next()
		.expect("should always have 1 effect in the effects map")
		.clone();

	DisplayStateEffect {
		config: effects
			.get(&default_effect)
			.unwrap()
			.default_config()
			.unwrap(),
		effect: default_effect,
	}
}

impl EffectRunner {
	pub fn new(config_dir: &Path, effects: EffectsMap, controller: Controller) -> Result<Self> {
		let config = Config::<GlobalConfig>::load(config_dir)?;
		let state = Config::<DisplayState>::load(config_dir)?;
		let presets = Config::<Presets>::load(config_dir)?;

		let mut runner = EffectRunner {
			effects,
			effects_state: vec![],
			controller,

			config,
			state,
			presets,

			state_notifier: channel(1).0,
			timer: Timer::new(),
			counter: 0,
		};
		runner.validate_state()?;

		Ok(runner)
	}

	pub fn validate_state(&mut self) -> Result<()> {
		if self.state.effects.is_empty() {
			self.state.effects = vec![default_effect(&self.effects)];
			let mut segments = vec![];

			for strip in self.config.strips.iter() {
				segments.push(vec![0; strip.segments.len()]);
			}
			self.state.segments = segments;
		}

		for effect in self.state.effects.iter_mut() {
			if self.effects.contains_key(&effect.effect) {
				continue;
			}

			*effect = default_effect(&self.effects)
		}

		for (i, strip) in self.state.segments.iter().enumerate() {
			let segments = if let Some(segments) = self.effects_state.get_mut(i) {
				segments
			} else {
				self.effects_state.push(Vec::with_capacity(strip.len()));
				self.effects_state
					.get_mut(i)
					.expect("previous iterations should make sure this works")
			};

			for (j, effect_ref) in strip.iter().enumerate() {
				let effect = &self.state.effects[*effect_ref];
				let factory = self.effects.get(&effect.effect).unwrap();

				if let Some((name, instance)) = segments.get_mut(j) {
					if *name != effect.effect {
						*instance = factory.build(effect.config.clone())?;
					} else {
						instance.set_config(effect.config.clone())?;
					}
				} else {
					segments.push((effect.effect.clone(), factory.build(effect.config.clone())?));
				}
			}

			while segments.len() > strip.len() {
				segments.pop();
			}
		}

		while self.state.segments.len() > self.effects_state.len() {
			self.effects_state.pop();
		}

		self.state.save()?;

		Ok(())
	}

	pub fn tick(&mut self) {
		for (i, strip_effects) in self.effects_state.iter_mut().enumerate() {
			let strip = &self.config.strips[i];

			let mut led_index = strip.offset;
			for (j, (_, instance)) in strip_effects.iter_mut().enumerate() {
				let segment = &strip.segments[j];

				let section =
					self.controller
						.section(i, led_index, segment.length, segment.reversed);
				instance.run(section);

				led_index += segment.length;
			}
		}

		self.controller.write_state(&self.config);

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
	fn get_global_config(&self) -> Result<&GlobalConfig> {
		Ok(&self.config)
	}

	fn set_global_config(&mut self, config: GlobalConfig) -> Result<()> {
		self.config.set(config);
		self.config.save()?;

		Ok(())
	}

	fn list_effects(&self) -> Result<HashMap<String, EffectData>> {
		let mut map = HashMap::with_capacity(self.effects.len());

		for (name, effect_factory) in self.effects.iter() {
			map.insert(
				name.clone(),
				EffectData {
					name:           name.clone(),
					schema:         effect_factory.schema(),
					default_config: effect_factory.default_config()?,
				},
			);
		}

		Ok(map)
	}

	fn list_presets(&self) -> Result<&HashMap<String, DisplayState>> {
		Ok(&self.presets.0)
	}

	fn set_preset(&mut self, name: String, preset: DisplayState) -> Result<()> {
		self.presets.0.insert(name, preset);
		self.presets.save()?;

		Ok(())
	}

	fn delete_preset(&mut self, name: String) -> Result<()> {
		self.presets.0.remove(&name);
		self.presets.save()?;

		Ok(())
	}

	fn load_preset(&mut self, name: String) -> Result<()> {
		let Some(state) = self.presets.0.get(&name) else {
			bail!("preset not found: {}", name);
		};
		self.set_state(state.clone())?;

		Ok(())
	}

	fn save_preset(&mut self, name: String) -> Result<()> {
		self.presets.0.insert(name, self.state.clone());
		self.presets.save()?;

		Ok(())
	}

	fn get_state(&self) -> Result<&DisplayState> {
		Ok(&self.state)
	}

	fn set_state(&mut self, state: DisplayState) -> Result<()> {
		self.state.set(state.clone());
		self.validate_state()?;
		self.state.save()?;

		// error only means there's noe receiver, we don't care if that's the case.
		self.state_notifier.send(state).ok();

		Ok(())
	}

	fn subscribe(&self) -> Receiver<DisplayState> {
		self.state_notifier.subscribe()
	}
}
