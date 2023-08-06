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

	// add_effect(&mut effect_map, db, "explosions", |db| Explosions::new(db))?;
	// add_effect(&mut effect_map, db, "flash_rainbow", |db| {
	// 	FlashRainbow::new(db)
	// })?;
	// add_effect(&mut effect_map, db, "flash_rainbow_noise", |db| {
	// 	FlashRainbowNoise::new(db)
	// })?;
	// add_effect(&mut effect_map, db, "flash_rainbow_random", |db| {
	// 	FlashRainbowRandom::new(db)
	// })?;
	// add_effect(&mut effect_map, db, "meteors", |db| Meteors::new(db))?;
	// add_effect(&mut effect_map, db, "moving_lights", |db| {
	// 	MovingLights::new(db)
	// })?;
	// add_effect(&mut effect_map, db, "police", |db| Police::new(db))?;
	// add_effect(&mut effect_map, db, "rainbow", |db| Rainbow::new(db))?;
	// add_effect(&mut effect_map, db, "random", |db| RandomNoise::new(db))?;
	// add_effect(&mut effect_map, db, "snake", |db| Snake::new(db))?;
	// add_effect(&mut effect_map, db, "solid", |db| Solid::new(db))?;
	// add_effect(&mut effect_map, db, "static_rainbow", |db| {
	// 	StaticRainbow::new(db)
	// })?;

	Ok(effect_map)
}
