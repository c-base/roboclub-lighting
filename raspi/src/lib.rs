use std::collections::HashMap;

use eyre::Result;
use serde::{de::DeserializeOwned, Serialize};
use utoipa::ToSchema;

use crate::effects::EffectFactory;

pub mod color;
pub mod config;
pub mod controller;
pub mod effects;
pub mod grpc;
pub mod http;
pub mod noise;
pub mod runner;
pub mod serde_transcode;

pub fn all_internal_effects() -> Result<HashMap<String, Box<dyn EffectFactory>>> {
	let mut effect_map: HashMap<String, Box<dyn EffectFactory>> = HashMap::new();

	fn add_effect<
		C: Default + Serialize + DeserializeOwned + for<'a> ToSchema<'a> + Send + Sync + 'static,
		S: Default + Send + Sync + 'static,
	>(
		map: &mut HashMap<String, Box<dyn EffectFactory>>,
		name: &str,
		func: impl EffectFn<C, S> + Send + Sync + Clone + 'static,
	) -> Result<()> {
		map.insert(name.to_string(), Box::new(FnEffectFactory::new(func)));
		Ok(())
	}

	use effects::*;

	add_effect(&mut effect_map, "balls", balls)?;
	add_effect(&mut effect_map, "solid", solid)?;
	add_effect(&mut effect_map, "static_rainbow", static_rainbow)?;
	add_effect(&mut effect_map, "explosions", explosions)?;
	add_effect(&mut effect_map, "flash_rainbow", flash_rainbow)?;
	add_effect(&mut effect_map, "flash_rainbow_noise", flash_rainbow_noise)?;
	add_effect(
		&mut effect_map,
		"flash_rainbow_random",
		flash_rainbow_random,
	)?;
	add_effect(&mut effect_map, "meteors", meteors)?;
	add_effect(&mut effect_map, "moving_lights", moving_lights)?;
	// add_effect(&mut effect_map, db, "police", |db| Police::new(db))?;
	add_effect(&mut effect_map, "rainbow", rainbow)?;
	add_effect(&mut effect_map, "random", random)?;
	add_effect(&mut effect_map, "snake", snake)?;

	Ok(effect_map)
}
