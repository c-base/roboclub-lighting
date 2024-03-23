use std::{
	env::current_dir,
	fs,
	sync::{Arc, Mutex},
	time::Duration,
};

use eyre::Result;
use robolab::{all_internal_effects, controller::Controller, grpc, http, runner::EffectRunner};
use tracing::info;

pub const APP_NAME: &str = "roboclub-led-controller";

fn install_tracing() {
	use tracing_error::ErrorLayer;
	use tracing_subscriber::{fmt, prelude::*, EnvFilter};

	// fmt::layer().pretty().compact().with_target(false);
	let fmt_layer = fmt::layer().with_target(false);
	let filter_layer = EnvFilter::try_from_default_env()
		.or_else(|_| EnvFilter::try_new("info"))
		.unwrap();

	tracing_subscriber::registry()
		.with(filter_layer)
		.with(fmt_layer)
		.with(ErrorLayer::default())
		.init();
}

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install()?;
	install_tracing();

	let config_dir = current_dir()?.join(".config");
	fs::create_dir_all(&config_dir)?;

	let controller = Controller::new()?;

	let runner = {
		let effect_map = all_internal_effects()?;

		let runner = EffectRunner::new(&config_dir, effect_map, controller)?;
		Arc::new(Mutex::new(runner))
	};

	let _handle = {
		let runner = runner.clone();
		std::thread::spawn(move || {
			info!("starting effect loop");
			loop {
				runner.lock().unwrap().tick();
				// std::thread::yield_now();
				std::thread::sleep(Duration::from_micros(100));
			}
		})
	};

	tokio::try_join!(http::run(runner.clone()), grpc::run(runner.clone()))?;

	// let mut io = IoHandler::default();
	// io.add_sync_method("say_hello", |_params| {
	// 	println!("Processing");
	// 	Ok(Value::String("hello".to_owned()))
	// });
	//
	// let server = ServerBuilder::new(io)
	// 	.start(&"0.0.0.0:3030".parse().unwrap())
	// 	.expect("Server must start with no issues");
	//
	// server.wait()

	// jsonrpc::start();

	Ok(())
}
