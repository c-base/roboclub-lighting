#![allow(unused)]

use std::{error::Error, time::Duration};

use futures::{FutureExt, StreamExt};
use rppal::spi::Bus;
use tracing::{debug, error, info, instrument, warn};
use tracing_futures::WithSubscriber;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter, FmtSubscriber};
use warp::Filter;

use crate::controller::Controller;

// mod audio;
mod colour;
mod controller;
mod effects;
mod noise;

pub const APP_NAME: &'static str = "roboclub-led-controller";

fn main() -> Result<(), Box<dyn Error>> {
	color_backtrace::install();

	let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_owned());

	let sub = FmtSubscriber::builder()
		.pretty()
		.compact()
		.with_env_filter(filter)
		.finish();

	tracing::subscriber::set_global_default(sub).expect("failed to start the logger.");

	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()?
		.block_on(start())
}

#[instrument]
async fn start() -> Result<(), Box<dyn Error>> {
	const GPIO_READY: u8 = 17;
	let controller = Controller::new(GPIO_READY, Bus::Spi3)?;

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

	let home = warp::fs::dir("public");

	let ws = warp::path("ws")
		// The `ws()` filter will prepare the Websocket handshake.
		.and(warp::ws())
		.map(|ws: warp::ws::Ws| {
			// And then our closure will be called when it completes...
			ws.on_upgrade(|websocket| {
				// Just echo all messages back...
				let (tx, rx) = websocket.split();
				rx.forward(tx).map(|result| {
					if let Err(e) = result {
						eprintln!("websocket error: {:?}", e);
					}
				})
			})
		});

	let routes = home.or(ws).with(warp::trace::request());

	let handle = tokio::runtime::Handle::current();

	let server = warp::serve(routes);
	// let (_, srv) = server.try_bind_with_graceful_shutdown(([0, 0, 0, 0], 3030), async {
	// 	tokio::signal::ctrl_c()
	// 		.await
	// 		.expect("failed to listen for event");
	// })?;

	handle.spawn_blocking(|| {
		// let step = effects::rainbow();
		// let step = effects::snake();
		// let step = effects::random();
		// let step = effects::flash_rainbow();
		let step = effects::explosions(250, 0.5, 0.99, 0.99);
		// let step = effects::police();
		// let step = effects::moving_lights(20, 15, 2000);
		// let step = effects::meteors(10, 16, 40, 4);
		// let step = effects::random();

		info!("starting effect loop");
		effects::run(controller, step);
	});

	// srv.await;
	server.run(([0, 0, 0, 0], 3030)).await;

	Ok(())
}

fn start_server() {}
