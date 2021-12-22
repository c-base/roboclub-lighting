#![feature(decl_macro)]
#![allow(unused)]

#[macro_use]
extern crate rocket;

use std::{
	collections::HashMap,
	error::Error,
	sync::{Arc, Mutex, RwLock},
	time::Duration,
};

use color_eyre::{eyre::WrapErr, Result};
// use jsonrpc_tcp_server::{jsonrpc_core::IoHandler, ServerBuilder};
use rppal::spi::Bus;
use tracing::info;
use tracing_subscriber::{util::SubscriberInitExt, FmtSubscriber};

use crate::{
	controller::{Controller, ControllerConfig},
	db::load_config,
	effects::*,
};

// mod audio;
mod color;
mod controller;
mod db;
mod effects;
mod http;
mod jsonrpc;
mod noise;
pub mod runner;

pub const APP_NAME: &'static str = "roboclub-led-controller";

fn install_tracing() {
	// use tracing_error::ErrorLayer;
	use tracing_subscriber::{fmt, prelude::*, EnvFilter};

	let filter = EnvFilter::try_from_default_env()
		.or_else(|_| EnvFilter::try_new("info"))
		.unwrap();

	FmtSubscriber::builder()
		.pretty()
		.compact()
		.with_env_filter(filter)
		.finish()
		.init();
}

fn main() -> Result<()> {
	color_backtrace::install();
	color_eyre::install()?;

	install_tracing();

	let mut db = sled::open("db").wrap_err("should be able to open sled db")?;

	// tokio::runtime::Builder::new_current_thread()
	// 		.enable_all()
	// 		.build()?
	// 		.block_on(start())
	// }
	//
	// #[instrument]
	// async fn start() -> Result<(), Box<dyn Error>> {

	const GPIO_READY: u8 = 0;

	let mut ctrl_db = db
		.open_tree("controller")
		.expect("should be able to open a tree");
	let ctrl_config = load_config(&mut ctrl_db);
	let mut controller = Controller::new(ctrl_config, GPIO_READY, Bus::Spi0)?;
	drop(ctrl_db);

	// let mut frames = audio::get_frames().unwrap();
	//
	// let mut last_beat = 0;
	// for frame in frames.iter() {
	// 	// log::trace!("Frame: {:7}@{:.3}", frame.frame, frame.time);
	//
	// 	frame.info(|info| {
	// 		// use sfml::graphics::Shape;
	//
	// 		let max = info.average.max();
	// 		let n50 = info.average.freq_to_id(50.0);
	// 		let n100 = info.average.freq_to_id(100.0);
	//
	// 		let beat = if info.beat > last_beat {
	// 			last_beat = info.beat;
	// 			// rectangle.set_fill_color(&graphics::Color::rgb(255, 255, 255));
	// 			true
	// 		} else {
	// 			false
	// 		};
	//
	// 		for (i, b) in info.average.iter().enumerate() {
	// 			// use sfml::graphics::Transformable;
	//
	// 			let int = ((b / max).sqrt() * 255.0) as u8;
	// 			if !beat {
	// 				// rectangle.set_fill_color(&graphics::Color::rgb(int, int, int));
	// 				if i == n50 || i == n100 {
	// 					// rectangle.set_fill_color(&graphics::Color::rgb(255, 0, 0));
	// 				}
	// 			}
	// 			// rectangle.set_position(system::Vector2f::new(
	// 			// 	i as f32 / BUCKETS as f32,
	// 			// 	LINES as f32 - 1.0,
	// 			// ));
	// 			// window.draw(&rectangle);
	// 		}
	// 	});
	//
	// 	// window.display();
	// 	std::thread::sleep(Duration::from_millis(10));
	// }

	// let home = warp::fs::dir("public");
	//
	// let ws = warp::path("ws")
	// 	// The `ws()` filter will prepare the Websocket handshake.
	// 	.and(warp::ws())
	// 	.map(|ws: warp::ws::Ws| {
	// 		// And then our closure will be called when it completes...
	// 		ws.on_upgrade(|websocket| {
	// 			// Just echo all messages back...
	// 			let (tx, rx) = websocket.split();
	// 			rx.forward(tx).map(|result| {
	// 				if let Err(e) = result {
	// 					eprintln!("websocket error: {:?}", e);
	// 				}
	// 			})
	// 		})
	// 	});
	//
	// let routes = home.or(ws).with(warp::trace::request());
	//
	// let handle = tokio::runtime::Handle::current();
	//
	// let server = warp::serve(routes);
	// let (_, srv) = server.try_bind_with_graceful_shutdown(([0, 0, 0, 0], 3030), async {
	// 	tokio::signal::ctrl_c()
	// 		.await
	// 		.expect("failed to listen for event");
	// })?;

	let runner = {
		let mut effect_map: HashMap<String, Box<dyn effects::Effect>> = HashMap::new();

		fn add_effect<E: Effect + 'static, T: FnOnce(sled::Tree) -> E>(
			map: &mut HashMap<String, Box<dyn effects::Effect>>,
			db: &mut sled::Db,
			name: &str,
			init: T,
		) -> Result<()> {
			let tree = db
				.open_tree(name)
				.wrap_err("should be able to open a tree")?;
			map.insert(name.to_string(), Box::new(init(tree)));
			Ok(())
		}

		use effects::*;

		add_effect(&mut effect_map, &mut db, "balls", |db| Balls::new(db))?;
		add_effect(&mut effect_map, &mut db, "explosions", |db| {
			Explosions::new(db)
		})?;
		add_effect(&mut effect_map, &mut db, "flash_rainbow", |db| {
			FlashRainbow::new(db)
		})?;
		add_effect(&mut effect_map, &mut db, "flash_rainbow_noise", |db| {
			FlashRainbowNoise::new(db)
		})?;
		add_effect(&mut effect_map, &mut db, "flash_rainbow_random", |db| {
			FlashRainbowRandom::new(db)
		})?;
		add_effect(&mut effect_map, &mut db, "meteors", |db| Meteors::new(db))?;
		add_effect(&mut effect_map, &mut db, "moving_lights", |db| {
			MovingLights::new(db)
		})?;
		add_effect(&mut effect_map, &mut db, "police", |db| Police::new(db))?;
		add_effect(&mut effect_map, &mut db, "rainbow", |db| Rainbow::new(db))?;
		add_effect(&mut effect_map, &mut db, "random", |db| {
			RandomNoise::new(db)
		})?;
		add_effect(&mut effect_map, &mut db, "snake", |db| Snake::new(db))?;
		add_effect(&mut effect_map, &mut db, "solid", |db| Solid::new(db))?;
		add_effect(&mut effect_map, &mut db, "static_rainbow", |db| {
			StaticRainbow::new(db)
		})?;

		let tree = db
			.open_tree("controller")
			.wrap_err("should be able to open a tree")?;
		let runner = runner::EffectRunner::new(tree, effect_map, controller);
		Arc::new(Mutex::new(runner))
	};

	let _handle = {
		let runner = runner.clone();
		std::thread::spawn(move || {
			info!("starting effect loop");
			loop {
				let mut runner = runner.lock().unwrap();
				runner.tick();
				drop(runner);
				// std::thread::yield_now();
				std::thread::sleep(Duration::from_micros(100));
			}
		})
	};

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
	http::run(runner.clone())?;

	Ok(())
}
