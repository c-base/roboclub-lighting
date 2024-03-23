use std::{
	collections::{HashMap, HashSet},
	path::Path,
};

use eyre::{bail, ContextCompat, Result, WrapErr};
use tokio::sync::broadcast::{channel, Receiver, Sender};
use tracing::{debug, error};

use crate::{
	config::{
		Config,
		DisplayState,
		DisplayStateEffect,
		GlobalConfig,
		Group,
		Presets,
		SegmentId,
		Strip,
	},
	controller::{Controller, LedController},
	effects::{prelude::Timer, Effect, EffectData, EffectFactory},
};

type EffectsMap = HashMap<String, Box<dyn EffectFactory>>;

#[derive(Clone, Debug, Hash, Ord, PartialOrd, PartialEq, Eq)]
enum EffectTarget {
	Segment(SegmentId),
	Group(String),
}

pub struct EffectRunner {
	effects:       EffectsMap,
	effects_state: HashMap<EffectTarget, (String, Box<dyn Effect>)>,
	controller:    Controller,

	config:  Config<GlobalConfig>,
	state:   Config<DisplayState>,
	presets: Config<Presets>,

	state_notifier: Sender<DisplayState>,
	timer:          Timer,
	counter:        usize,
}

#[derive(Clone, Debug, Default)]
pub struct ApiConfig {
	pub brightness: f32,
	pub srgb:       bool,
}

pub trait EffectAPI {
	fn get_global_config(&self) -> Result<ApiConfig>;
	fn set_global_config(&mut self, config: ApiConfig) -> Result<()>;

	fn list_segments(&self) -> Result<Vec<Strip>>;
	fn set_segments(&mut self, strips: Vec<Strip>) -> Result<()>;
	fn list_groups(&self) -> Result<Vec<Group>>;
	fn set_groups(&mut self, config: Vec<Group>) -> Result<()>;

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
		config:      effects
			.get(&default_effect)
			.unwrap()
			.default_config()
			.unwrap(),
		effect_id:   default_effect,
		segment_ids: HashSet::new(),
		group_ids:   HashSet::new(),
	}
}

impl EffectRunner {
	pub fn new(config_dir: &Path, effects: EffectsMap, controller: Controller) -> Result<Self> {
		let config = Config::<GlobalConfig>::load(config_dir)?;
		let state = Config::<DisplayState>::load(config_dir)?;
		let presets = Config::<Presets>::load(config_dir)?;

		let mut runner = EffectRunner {
			effects,
			effects_state: HashMap::new(),
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

	#[tracing::instrument(skip(self))]
	pub fn validate_config(&mut self) -> Result<()> {
		let GlobalConfig { strips, groups, .. } = &mut *self.config;

		let ctrl_state = &*self.controller.state_mut();

		if strips.len() > ctrl_state.len() {
			error!(
				"configured strips are longer than supported ({} > {}), reducing",
				strips.len(),
				ctrl_state.len(),
			);

			strips.drain(ctrl_state.len()..);
		}

		for (strip_idx, strip) in strips.iter_mut().enumerate() {
			let mut led_idx = strip.offset;

			for (idx, segment) in strip.segments.clone().into_iter().enumerate() {
				if led_idx + segment.length >= ctrl_state[0].len() {
					error!(
						"configured strip {} segment {} ({}) goes over the max number of LEDs ({} > {}), reducing and dropping any additional segments",
						strip_idx,
						idx,
						segment.name,
						led_idx + segment.length,
						ctrl_state.len()
					);

					strip.segments.drain(idx..);

					break;
				}

				led_idx += segment.length;
			}
		}

		for group in groups.iter_mut() {
			for segment_id in group.segment_ids.clone() {
				let Some(strip) = strips.get(segment_id.strip_idx) else {
					error!(
						"group {} is referencing an invalid strip {} (removed now)",
						group.name, segment_id.strip_idx
					);

					group.segment_ids.remove(&segment_id);
					continue;
				};

				if strip.segments.get(segment_id.segment_idx).is_none() {
					error!(
						"group {} is referencing an invalid segment {} of strip {} (removed now)",
						group.name, segment_id.segment_idx, segment_id.strip_idx
					);

					group.segment_ids.remove(&segment_id);
					continue;
				}
			}
		}

		self.config.save()?;

		Ok(())
	}

	#[tracing::instrument(skip(self))]
	pub fn validate_state(&mut self) -> Result<()> {
		if self.state.effects.is_empty() {
			let mut effect = default_effect(&self.effects);

			for (i, strip) in self.config.strips.iter().enumerate() {
				for (j, _) in strip.segments.iter().enumerate() {
					effect.segment_ids.insert(SegmentId {
						strip_idx:   i,
						segment_idx: j,
					});
				}
			}

			self.state.effects = vec![effect];
		}

		for effect in self.state.effects.iter_mut() {
			if !self.effects.contains_key(&effect.effect_id) {
				let default = default_effect(&self.effects);

				effect.effect_id = default.effect_id;
				effect.config = default.config;
			}

			let mut targets: Vec<EffectTarget> = vec![];

			for segment_id in effect.segment_ids.clone() {
				let Some(strip) = self.config.strips.get(segment_id.strip_idx) else {
					error!(
						"effect {} is referencing an invalid strip {}",
						effect.effect_id, segment_id.strip_idx
					);

					effect.segment_ids.remove(&segment_id);
					continue;
				};

				if strip.segments.get(segment_id.segment_idx).is_none() {
					error!(
						"effect {} is referencing an invalid segment {} of strip {}",
						effect.effect_id, segment_id.segment_idx, segment_id.strip_idx
					);

					effect.segment_ids.remove(&segment_id);
					continue;
				}

				targets.push(EffectTarget::Segment(segment_id));
			}

			for group_id in effect.group_ids.clone() {
				if self.config.groups.iter().any(|group| group.id == group_id) {
					error!(
						"effect {} is referencing group {} not found in the config",
						effect.effect_id, group_id,
					);

					effect.group_ids.remove(&group_id);
					continue;
				};

				targets.push(EffectTarget::Group(group_id));
			}

			for effect_target in targets {
				let factory = self
					.effects
					.get(&effect.effect_id)
					.wrap_err("effect factory should exist")?;

				if let Some((effect_id, instance)) = self.effects_state.get_mut(&effect_target) {
					if *effect_id != effect.effect_id {
						*instance = factory.build(effect.config.clone())?;
						effect_id.clone_from(&effect.effect_id);
					} else {
						instance
							.set_config(effect.config.clone())
							.wrap_err_with(|| format!("setting config for effect {}", effect_id))?;
					}
				} else {
					let factory = self.effects.get(&effect.effect_id).unwrap();

					self.effects_state.insert(
						effect_target,
						(
							effect.effect_id.clone(),
							factory.build(effect.config.clone())?,
						),
					);
				}
			}
		}

		self.state.save()?;

		Ok(())
	}

	pub fn tick(&mut self) {
		for (target, (name, instance)) in self.effects_state.iter_mut() {
			let segments_ids = match target {
				EffectTarget::Segment(segment_id) => vec![*segment_id],
				EffectTarget::Group(group_id) => {
					let Some(group) = self
						.config
						.groups
						.iter()
						.find(|group| group.id == *group_id)
					else {
						error!(
							"effect {} is referencing group {} not found in the config",
							name, group_id,
						);
						continue;
					};

					group.segment_ids.iter().copied().collect()
				}
			};

			for segment_id in segments_ids {
				let Some(strip) = self.config.strips.get(segment_id.strip_idx) else {
					error!(
						"effect {} is referencing an invalid strip {}",
						name, segment_id.strip_idx
					);
					continue;
				};

				let Some(segment) = strip.segments.get(segment_id.segment_idx) else {
					error!(
						"effect {} is referencing an invalid segment {} of strip {}",
						name, segment_id.segment_idx, segment_id.strip_idx
					);
					continue;
				};

				let mut led_idx = strip.offset;
				for segment in strip.segments.iter().take(segment_id.segment_idx) {
					led_idx += segment.length;
				}

				let section = self.controller.section(
					segment_id.strip_idx,
					led_idx,
					segment.length,
					segment.reversed,
				);

				instance.run(section);
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
	#[tracing::instrument(skip(self))]
	fn get_global_config(&self) -> Result<ApiConfig> {
		Ok(ApiConfig {
			brightness: self.config.brightness,
			srgb:       self.config.as_srgb,
		})
	}

	#[tracing::instrument(skip(self))]
	fn set_global_config(&mut self, config: ApiConfig) -> Result<()> {
		self.config.brightness = config.brightness;
		self.config.as_srgb = config.srgb;
		self.config.save()?;

		Ok(())
	}

	#[tracing::instrument(skip(self))]
	fn list_segments(&self) -> Result<Vec<Strip>> {
		return Ok(self.config.strips.clone());
	}

	#[tracing::instrument(skip(self, strips))]
	fn set_segments(&mut self, strips: Vec<Strip>) -> Result<()> {
		self.config.strips = strips;
		self.validate_state()?;
		self.config.save()?;

		Ok(())
	}

	#[tracing::instrument(skip(self))]
	fn list_groups(&self) -> Result<Vec<Group>> {
		return Ok(self.config.groups.clone());
	}

	#[tracing::instrument(skip(self, groups))]
	fn set_groups(&mut self, groups: Vec<Group>) -> Result<()> {
		self.config.groups = groups;
		self.validate_state()?;
		self.config.save()?;

		Ok(())
	}

	#[tracing::instrument(skip(self))]
	fn list_effects(&self) -> Result<HashMap<String, EffectData>> {
		let mut map = HashMap::with_capacity(self.effects.len());

		for (name, effect_factory) in self.effects.iter() {
			map.insert(
				name.clone(),
				EffectData {
					// TODO: add an actual ID?
					id:             name.clone(),
					name:           name.clone(),
					schema:         effect_factory.schema(),
					default_config: effect_factory.default_config()?,
				},
			);
		}

		Ok(map)
	}

	#[tracing::instrument(skip(self))]
	fn list_presets(&self) -> Result<&HashMap<String, DisplayState>> {
		Ok(&self.presets.0)
	}

	#[tracing::instrument(skip(self, preset))]
	fn set_preset(&mut self, name: String, preset: DisplayState) -> Result<()> {
		self.presets.0.insert(name, preset);
		self.presets.save()?;

		Ok(())
	}

	#[tracing::instrument(skip(self))]
	fn delete_preset(&mut self, name: String) -> Result<()> {
		self.presets.0.remove(&name);
		self.presets.save()?;

		Ok(())
	}

	#[tracing::instrument(skip(self))]
	fn load_preset(&mut self, name: String) -> Result<()> {
		let Some(state) = self.presets.0.get(&name) else {
			bail!("preset not found: {}", name);
		};
		self.set_state(state.clone())?;

		Ok(())
	}

	#[tracing::instrument(skip(self))]
	fn save_preset(&mut self, name: String) -> Result<()> {
		self.presets.0.insert(name, self.state.clone());
		self.presets.save()?;

		Ok(())
	}

	#[tracing::instrument(skip(self))]
	fn get_state(&self) -> Result<&DisplayState> {
		Ok(&self.state)
	}

	#[tracing::instrument(skip(self, state))]
	fn set_state(&mut self, state: DisplayState) -> Result<()> {
		self.state.set(state.clone());
		self.validate_state()?;
		self.state.save()?;

		// error only means there's no receiver, we don't care if that's the case.
		self.state_notifier.send(state).ok();

		Ok(())
	}

	fn subscribe(&self) -> Receiver<DisplayState> {
		self.state_notifier.subscribe()
	}
}
