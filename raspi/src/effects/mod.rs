use std::{fmt::Debug, marker::PhantomData};

use eyre::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use utoipa::{
	openapi::{RefOr, Schema},
	ToSchema,
};

pub use crate::effects::balls::balls;
use crate::{config::WithConfig, controller::Section};

pub mod balls;
pub mod config;
// pub mod explosions;
// pub mod flash_rainbow;
// pub mod flash_rainbow_noise;
// pub mod flash_rainbow_random;
// pub mod meteors;
// pub mod moving_lights;
// pub mod police;
pub mod prelude;
// pub mod rainbow;
// pub mod random;
// pub mod schema;
// pub mod snake;
// pub mod solid;
// pub mod static_rainbow;

pub trait EffectFactory: Send {
	fn schema(&self) -> Schema;
	fn default_config(&self) -> Result<serde_json::Value>;
	fn build(&self, config: serde_json::Value) -> Result<Box<dyn Effect>>;
}

pub struct FnEffectFactory<C, S, F> {
	func: F,

	_marker: PhantomData<fn() -> (C, S)>,
}

impl<C, S, F> FnEffectFactory<C, S, F> {
	pub fn new(func: F) -> Self {
		Self {
			func,
			_marker: PhantomData,
		}
	}
}

impl<C, S, F> EffectFactory for FnEffectFactory<C, S, F>
where
	C: Default + Serialize + DeserializeOwned + for<'a> ToSchema<'a> + Send + Sync + 'static,
	S: Default + Send + Sync + 'static,
	F: EffectFn<C, S> + Send + Sync + Clone + 'static,
{
	fn schema(&self) -> Schema {
		match C::schema().1 {
			RefOr::Ref(_) => {
				panic!(
					"EffectFactory::schema needs to return a schema, {}::schema() returned a ref",
					stringify!($config)
				)
			}
			RefOr::T(s) => s,
		}
	}

	fn default_config(&self) -> Result<serde_json::Value> {
		Ok(serde_json::to_value(C::default())?)
	}

	fn build(&self, config: serde_json::Value) -> Result<Box<dyn Effect>> {
		Ok(Box::new(EffectState::new(self.func.clone(), config)?))
	}
}

type EffectWindow<'a> = Section<'a>;
// pub enum EffectWindow<'a> {
// 	Single(Section<'a>),
// }

pub trait Effect: WithConfig<Config = serde_json::Value> + Send + Sync {
	fn run(&mut self, window: EffectWindow);
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct EffectData {
	pub name:           String,
	pub schema:         Schema,
	pub default_config: serde_json::Value,
}

pub trait EffectFn<C, S> {
	fn call(&self, config: &C, state: &mut S, window: EffectWindow);
}

impl<C, S, F> EffectFn<C, S> for F
where
	C: Default,
	S: Default,
	F: Fn(&C, &mut S, EffectWindow) + Send + Sync,
{
	fn call(&self, config: &C, state: &mut S, window: EffectWindow) {
		(self)(config, state, window)
	}
}

// impl<C, F> EffectFn<C, ()> for F
// where
// 	C: Default,
// 	F: Fn(&C, &mut Controller) + Send + Sync,
// {
// 	fn call(config: &C, _: &(), ctrl: &mut Controller) {
// 		(Self)(config, ctrl)
// 	}
// }
//
// impl<S, F> EffectFn<(), S> for F
// where
// 	S: Default,
// 	F: Fn(&mut S, &mut Controller) + Send + Sync,
// {
// 	fn call(_: &(), state: &mut S, ctrl: &mut Controller) {
// 		(Self)(state, ctrl)
// 	}
// }
//
// impl<F> EffectFn<(), ()> for F
// where
// 	F: Fn(&mut Controller) + Send + Sync,
// {
// 	fn call(_: &(), _: &(), ctrl: &mut Controller) {
// 		(Self)(ctrl)
// 	}
// }

struct EffectState<C, S, F>
where
	F: EffectFn<C, S>,
{
	config: C,
	state:  S,
	func:   F,
}

impl<C, S, F> EffectState<C, S, F>
where
	C: Default + DeserializeOwned,
	S: Default,
	F: EffectFn<C, S>,
{
	fn new(func: F, config: serde_json::Value) -> Result<Self> {
		let config = serde_json::from_value(config)?;

		Ok(Self {
			config,
			state: Default::default(),
			func,
		})
	}
}

impl<C, S, F> WithConfig for EffectState<C, S, F>
where
	C: Default + Serialize + DeserializeOwned + for<'a> ToSchema<'a> + Send + Sync,
	S: Default + Send + Sync,
	F: EffectFn<C, S> + Send + Sync,
{
	type Config = serde_json::Value;

	fn set_config(&mut self, value: Self::Config) -> Result<()> {
		// TODO: validation
		self.config = serde_json::from_value(value)?;
		Ok(())
	}
}

impl<C, S, F> Effect for EffectState<C, S, F>
where
	C: Default + Serialize + DeserializeOwned + for<'a> ToSchema<'a> + Send + Sync,
	S: Default + Send + Sync,
	F: EffectFn<C, S> + Send + Sync,
{
	fn run(&mut self, window: EffectWindow) {
		self.func.call(&self.config, &mut self.state, window)
	}
}

// #[macro_export]
// macro_rules! effect {
// 	($struct:ident, $config:ident) => {
// 		impl crate::effects::WithConfig for $struct {
// 			type Config = serde_json::Value;
//
// 			fn set_config(&mut self, value: Self::Config) -> eyre::Result<()> {
// 				let config: $config = serde_json::from_value(value)?;
// 				$struct::set_config(self, config);
// 				crate::config::db::save_config(&mut self.db, &self.config)?;
// 				Ok(())
// 			}
// 		}
//
// 		impl crate::effects::Effect for $struct {
// 			fn schema(&self) -> utoipa::openapi::Schema {
// 				match <$config as utoipa::ToSchema>::schema().1 {
// 					utoipa::openapi::RefOr::Ref(_) => {
// 						panic!(
// 							"Effect::schema needs to return a schema, {}::schema() returned a ref",
// 							stringify!($config)
// 						)
// 					}
// 					utoipa::openapi::RefOr::T(s) => s,
// 				}
// 			}
//
// 			fn run(&mut self, ctrl: &mut crate::controller::Controller) {
// 				$struct::run(self, ctrl);
// 			}
// 		}
// 	};
// }
