use std::{
	error::Error,
	sync::{Arc, RwLock},
};

use rocket::{config::Environment, response::NamedFile, Config, State};
use rocket_contrib::{json::Json, serve::StaticFiles};
use rocket_cors::CorsOptions;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::effects::{runner::EffectRunner, EffectData};

#[derive(Serialize)]
struct Context {}

#[get("/")]
fn index() -> Option<NamedFile> {
	NamedFile::open("public/index.html").ok()
}

#[derive(Serialize)]
struct Effects {
	active_effect: EffectData,
	effects:       Vec<EffectData>,
}

#[get("/api/effects")]
fn effects(
	runner: State<Arc<RwLock<EffectRunner>>>,
) -> Result<Json<Effects>, Box<dyn std::error::Error>> {
	let runner = runner.read().unwrap();

	Ok(Json(Effects {
		active_effect: runner.active_effect()?,
		effects:       runner.effects()?,
	}))
}

#[derive(Deserialize)]
struct ActiveEffectPayload {
	active_effect: String,
}

#[post("/api/active_effect", data = "<active_effect>")]
fn set_active_effect(
	active_effect: Json<ActiveEffectPayload>,
	runner: State<Arc<RwLock<EffectRunner>>>,
) -> Result<Json<EffectData>, Box<dyn std::error::Error>> {
	let mut runner = runner.write().unwrap();

	runner.set_active_effect(active_effect.active_effect.clone());

	Ok(Json(runner.active_effect()?))
}

#[put("/api/effects/<effect>", data = "<config>")]
fn set_effect_config(
	effect: String,
	config: Json<serde_json::Value>,
	runner: State<Arc<RwLock<EffectRunner>>>,
) -> Result<Json<EffectData>, Box<dyn std::error::Error>> {
	let mut runner = runner.write().unwrap();

	let data = runner.set_effect_config(effect, config.into_inner())?;

	Ok(Json(data))
}

pub(crate) fn run(runner: Arc<RwLock<EffectRunner>>) -> Result<(), Box<dyn Error>> {
	let cors = CorsOptions::default().send_wildcard(true).to_cors()?;

	let config = Config::build(Environment::Production)
		.address("0.0.0.0")
		.port(4444)
		.finalize()?;

	let rkt = rocket::custom(config)
		.mount("/", StaticFiles::from("public/"))
		.mount(
			"/",
			routes![index, effects, set_active_effect, set_effect_config],
		)
		.attach(cors)
		.manage(runner);

	let err = rkt.launch();
	error!("server died: {:?}", err);
	Err(Box::new(err))
}
