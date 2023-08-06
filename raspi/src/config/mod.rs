use std::{
	collections::HashMap,
	fs::File,
	io::{BufReader, BufWriter},
	ops::{Deref, DerefMut},
	path::{Path, PathBuf},
};

use educe::Educe;
use eyre::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub mod db;

pub trait WithConfig {
	type Config: Default + DeserializeOwned + Serialize;

	fn set_config(&mut self, config: Self::Config) -> Result<()>;
}

pub trait ConfigFile: Serialize + DeserializeOwned {
	fn path(config_dir: &Path) -> PathBuf;
}

pub struct Config<T> {
	inner: T,
	path:  PathBuf,
}

impl<T> Deref for Config<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<T> DerefMut for Config<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

impl<T: ConfigFile + Serialize + Default> Config<T> {
	pub fn load(config_dir: &Path) -> Result<Self> {
		let path = T::path(config_dir);

		if !path.exists() {
			let cfg = Self {
				inner: Default::default(),
				path,
			};
			cfg.save()?;

			return Ok(cfg);
		}

		let file = File::open(&path)?;
		let reader = BufReader::new(file);

		let config = serde_json::from_reader(reader)?;

		Ok(Self {
			inner: config,
			path,
		})
	}

	pub fn save(&self) -> Result<()> {
		let file = File::create(&self.path)?;
		let writer = BufWriter::new(file);

		serde_json::to_writer_pretty(writer, &self.inner)?;

		Ok(())
	}

	pub fn set(&mut self, val: T) {
		self.inner = val;
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, Educe)]
#[educe(Default)]
pub struct GlobalConfig {
	#[educe(Default = 1.0)]
	pub brightness: f32,
	#[educe(Default = false)]
	pub as_srgb:    bool,

	pub strips: Vec<Strip>,
}

impl ConfigFile for GlobalConfig {
	fn path(config_dir: &Path) -> PathBuf {
		config_dir.join("config.json")
	}
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Strip {
	pub offset:   usize,
	pub segments: Vec<Segment>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Segment {
	pub name:     String,
	pub length:   usize,
	pub reversed: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Presets(pub HashMap<String, DisplayState>);

impl ConfigFile for Presets {
	fn path(config_dir: &Path) -> PathBuf {
		config_dir.join("presets.json")
	}
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DisplayState {
	pub effects:  Vec<DisplayStateEffect>,
	pub segments: Vec<Vec<usize>>,
}

impl ConfigFile for DisplayState {
	fn path(config_dir: &Path) -> PathBuf {
		config_dir.join("state.json")
	}
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DisplayStateEffect {
	pub effect: String,
	pub config: serde_json::Value,
}
