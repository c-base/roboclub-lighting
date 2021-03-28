use std::fmt::Debug;

use anyhow::Context;
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, error};

pub const CONFIG_KEY: &'static str = "config";

pub fn save_json<T: Serialize + Debug, K: AsRef<[u8]>>(
	db: &mut sled::Tree,
	key: K,
	value: &T,
) -> anyhow::Result<()> {
	let vec =
		serde_json::to_vec(value).with_context(|| format!("failed serializing {:?}", value))?;
	db.insert(key, vec)
		.with_context(|| "failed to write config to db")?;
	Ok(())
}

pub fn load_json<T: DeserializeOwned, K: AsRef<[u8]>>(
	db: &sled::Tree,
	key: K,
) -> anyhow::Result<Option<T>> {
	let vec = match db.get(key)? {
		Some(vec) => vec,
		None => {
			debug!("config not found in db, creating default");
			return Ok(None);
		}
	};

	let value = serde_json::from_slice(&*vec)?;
	return Ok(value);
}

pub fn save_effect_config<T: Serialize + Debug>(
	db: &mut sled::Tree,
	config: &T,
) -> anyhow::Result<()> {
	save_json(db, CONFIG_KEY, config)
}

pub fn load_effect_config<T: DeserializeOwned + Default>(db: &sled::Tree) -> T {
	match load_json(db, CONFIG_KEY) {
		Ok(opt) => match opt {
			Some(cfg) => cfg,
			None => {
				debug!("config not found in db, creating default");
				Default::default()
			}
		},
		Err(err) => {
			error!(
				"creating default config: failed to deserialize config from db: {}",
				err
			);
			Default::default()
		}
	}
}
