use std::fmt::Debug;

use eyre::{Result, WrapErr};
use serde::{de::DeserializeOwned, Serialize};
use tracing::{debug, error};

pub const CONFIG_KEY: &str = "config";

pub fn save_json<T: Serialize + Debug, K: AsRef<[u8]>>(
	db: &mut sled::Tree,
	key: K,
	value: &T,
) -> Result<()> {
	let vec =
		serde_json::to_vec(value).wrap_err_with(|| format!("failed serializing {:?}", value))?;
	db.insert(key, vec)
		.wrap_err("failed to write config to db")?;
	Ok(())
}

pub fn load_json<T: DeserializeOwned, K: AsRef<[u8]>>(
	db: &sled::Tree,
	key: K,
) -> Result<Option<T>> {
	let vec = match db.get(key)? {
		Some(vec) => vec,
		None => {
			debug!("config not found in db, creating default");
			return Ok(None);
		}
	};

	let value = serde_json::from_slice(&vec)?;
	Ok(value)
}

pub fn save_config<T: Serialize + Debug>(db: &mut sled::Tree, config: &T) -> Result<()> {
	save_json(db, CONFIG_KEY, config)
}

pub fn load_config<T: Serialize + DeserializeOwned + Debug + Default>(db: &mut sled::Tree) -> T {
	match load_json(db, CONFIG_KEY) {
		Ok(opt) => match opt {
			Some(cfg) => cfg,
			None => {
				debug!("config not found in db `{:?}`, creating default", db.name());
				let cfg: T = Default::default();
				save_config::<T>(db, &cfg).ok();
				cfg
			}
		},
		Err(err) => {
			error!(
				"creating default config: failed to deserialize config from db `{:?}`: {}",
				db.name(),
				err
			);
			let cfg: T = Default::default();
			save_config::<T>(db, &cfg).ok();
			cfg
		}
	}
}
