use std::{
	net::SocketAddr,
	sync::{Arc, Mutex},
};

use axum::Router;
use eyre::Result;
use tower_http::{cors::CorsLayer, services::ServeDir};

use crate::{
	grpc::{schema::controller_server::ControllerServer, MyController},
	runner::EffectRunner,
};

#[derive(Clone)]
struct AppState {
	// runner: Arc<Mutex<EffectRunner>>,
}
//
// // Make our own error that wraps `anyhow::Error`.
// struct AppError(eyre::Error);
//
// // Tell axum how to convert `AppError` into a response.
// impl IntoResponse for AppError {
// 	fn into_response(self) -> Response {
// 		(
// 			StatusCode::INTERNAL_SERVER_ERROR,
// 			format!("Something went wrong: {}", self.0),
// 		)
// 			.into_response()
// 	}
// }
//
// // This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// // `Result<_, AppError>`. That way you don't need to do that manually.
// impl<E> From<E> for AppError
// where
// 	E: Into<eyre::Error>,
// {
// 	fn from(err: E) -> Self {
// 		Self(err.into())
// 	}
// }
//
// #[derive(Serialize)]
// struct Effects {
// 	active_effect: String,
// 	effects:       HashMap<String, EffectData>,
// }
//
// #[axum::debug_handler]
// async fn effects(State(state): State<AppState>) -> Result<Json<Effects>, AppError> {
// 	let runner = state.runner.lock().unwrap();
//
// 	Ok(Json(Effects {
// 		active_effect: runner.get_active_effect()?.name.clone(),
// 		effects:       runner.get_effects()?,
// 	}))
// }
//
// #[derive(Deserialize)]
// struct ActiveEffectPayload {
// 	active_effect: String,
// }

// #[axum::debug_handler]
// async fn set_active_effect(
// 	State(state): State<AppState>,
// 	Json(active_effect): Json<ActiveEffectPayload>,
// ) -> Result<Json<EffectData>, AppError> {
// 	let mut runner = state.runner.lock().unwrap();
//
// 	runner.set_active_effect(active_effect.active_effect.clone())?;
//
// 	Ok(Json(runner.get_active_effect()?))
// }
//
// #[axum::debug_handler]
// async fn set_effect_config(
// 	Path(effect): Path<String>,
// 	State(state): State<AppState>,
// 	Json(config): Json<serde_json::Value>,
// ) -> Result<Json<EffectData>, AppError> {
// 	let mut runner = state.runner.lock().unwrap();
//
// 	let data = runner.set_effect_config(effect, config)?;
//
// 	Ok(Json(data))
// }

// // TODO
// async fn config() {}
// async fn set_config() {}
// async fn segments() {}
// async fn set_segments() {}
// async fn current() {}
// async fn set_current() {}
// async fn presets() {}
// async fn preset() {}
// async fn set_preset() {}
// async fn load_preset() {}

pub async fn run(runner: Arc<Mutex<EffectRunner>>) -> Result<()> {
	let controller = MyController {
		runner: runner.clone(),
	};
	let controller = ControllerServer::new(controller);

	let app = Router::new()
		.fallback_service(ServeDir::new("public/").append_index_html_on_directories(true))
		// .route("/api/config", get(config).put(set_config))
		// .route("/api/config/segments", get(segments).put(set_segments))
		// .route("/api/current", get(current).put(set_current))
		// .route("/api/effects", get(effects))
		// .route("/api/load_preset", post(load_preset))
		// .route("/api/presets", get(presets))
		// .route("/api/presets/:preset", get(preset).put(set_preset))
		// .route("/ws", get(ws_handler))
		.layer(CorsLayer::permissive())
		// .layer(TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default()))
		.route_service("/api/grpc", controller)
		.with_state(AppState {});

	let addr = SocketAddr::from(([0, 0, 0, 0], 4444));
	tracing::debug!("http listening on {}", addr);

	let listener = tokio::net::TcpListener::bind(addr).await?;
	axum::serve(listener, app).await?;

	Ok(())
}

// /// The handler for the HTTP request (this gets called when the HTTP GET lands at the start
// /// of websocket negotiation). After this completes, the actual switching from HTTP to
// /// websocket protocol will occur.
// /// This is the last point where we can extract TCP/IP metadata such as IP address of the client
// /// as well as things from HTTP headers such as user-agent of the browser etc.
// async fn ws_handler(
// 	ws: WebSocketUpgrade,
// 	user_agent: Option<TypedHeader<headers::UserAgent>>,
// 	ConnectInfo(addr): ConnectInfo<SocketAddr>,
// ) -> impl IntoResponse {
// 	let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
// 		user_agent.to_string()
// 	} else {
// 		String::from("Unknown browser")
// 	};
// 	println!("`{user_agent}` at {addr} connected.");
// 	// finalize the upgrade process by returning upgrade callback.
// 	// we can customize the callback by sending additional info such as address.
// 	ws.on_upgrade(move |socket| handle_socket(socket, addr))
// }
//
// /// Actual websocket statemachine (one will be spawned per connection)
// async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
// 	//send a ping (unsupported by some browsers) just to kick things off and get a response
// 	if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
// 		println!("Pinged {}...", who);
// 	} else {
// 		println!("Could not send ping {}!", who);
// 		// no Error here since the only thing we can do is to close the connection.
// 		// If we can not send messages, there is no way to salvage the statemachine anyway.
// 		return;
// 	}
//
// 	// receive single message from a client (we can either receive or send with socket).
// 	// this will likely be the Pong for our Ping or a hello message from client.
// 	// waiting for message from a client will block this task, but will not block other client's
// 	// connections.
// 	if let Some(msg) = socket.recv().await {
// 		if let Ok(msg) = msg {
// 			if process_message(msg, who).is_break() {
// 				return;
// 			}
// 		} else {
// 			println!("client {who} abruptly disconnected");
// 			return;
// 		}
// 	}
//
// 	// Since each client gets individual statemachine, we can pause handling
// 	// when necessary to wait for some external event (in this case illustrated by sleeping).
// 	// Waiting for this client to finish getting its greetings does not prevent other clients from
// 	// connecting to server and receiving their greetings.
// 	for i in 1..5 {
// 		if socket
// 			.send(Message::Text(format!("Hi {i} times!")))
// 			.await
// 			.is_err()
// 		{
// 			println!("client {who} abruptly disconnected");
// 			return;
// 		}
// 		tokio::time::sleep(std::time::Duration::from_millis(100)).await;
// 	}
//
// 	// By splitting socket we can send and receive at the same time. In this example we will send
// 	// unsolicited messages to client based on some sort of server's internal event (i.e .timer).
// 	let (mut sender, mut receiver) = socket.split();
//
// 	// Spawn a task that will push several messages to the client (does not matter what client does)
// 	let mut send_task = tokio::spawn(async move {
// 		let n_msg = 20;
// 		for i in 0..n_msg {
// 			// In case of any websocket error, we exit.
// 			if sender
// 				.send(Message::Text(format!("Server message {i} ...")))
// 				.await
// 				.is_err()
// 			{
// 				return i;
// 			}
//
// 			tokio::time::sleep(std::time::Duration::from_millis(300)).await;
// 		}
//
// 		println!("Sending close to {who}...");
// 		if let Err(e) = sender
// 			.send(Message::Close(Some(CloseFrame {
// 				code:   axum::extract::ws::close_code::NORMAL,
// 				reason: Cow::from("Goodbye"),
// 			})))
// 			.await
// 		{
// 			println!("Could not send Close due to {}, probably it is ok?", e);
// 		}
// 		n_msg
// 	});
//
// 	// This second task will receive messages from client and print them on server console
// 	let mut recv_task = tokio::spawn(async move {
// 		let mut cnt = 0;
// 		while let Some(Ok(msg)) = receiver.next().await {
// 			cnt += 1;
// 			// print message and break if instructed to do so
// 			if process_message(msg, who).is_break() {
// 				break;
// 			}
// 		}
// 		cnt
// 	});
//
// 	// If any one of the tasks exit, abort the other.
// 	tokio::select! {
// 		rv_a = (&mut send_task) => {
// 			match rv_a {
// 				Ok(a) => println!("{} messages sent to {}", a, who),
// 				Err(a) => println!("Error sending messages {:?}", a)
// 			}
// 			recv_task.abort();
// 		},
// 		rv_b = (&mut recv_task) => {
// 			match rv_b {
// 				Ok(b) => println!("Received {} messages", b),
// 				Err(b) => println!("Error receiving messages {:?}", b)
// 			}
// 			send_task.abort();
// 		}
// 	}
//
// 	// returning from the handler closes the websocket connection
// 	println!("Websocket context {} destroyed", who);
// }
//
// /// helper to print contents of messages to stdout. Has special treatment for Close.
// fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
// 	match msg {
// 		Message::Text(t) => {
// 			println!(">>> {} sent str: {:?}", who, t);
// 		}
// 		Message::Binary(d) => {
// 			println!(">>> {} sent {} bytes: {:?}", who, d.len(), d);
// 		}
// 		Message::Close(c) => {
// 			if let Some(cf) = c {
// 				println!(
// 					">>> {} sent close with code {} and reason `{}`",
// 					who, cf.code, cf.reason
// 				);
// 			} else {
// 				println!(">>> {} somehow sent close message without CloseFrame", who);
// 			}
// 			return ControlFlow::Break(());
// 		}
//
// 		Message::Pong(v) => {
// 			println!(">>> {} sent pong with {:?}", who, v);
// 		}
// 		// You should never need to manually handle Message::Ping, as axum's websocket library
// 		// will do so for you automagically by replying with Pong and copying the v according to
// 		// spec. But if you need the contents of the pings you can see them here.
// 		Message::Ping(v) => {
// 			println!(">>> {} sent ping with {:?}", who, v);
// 		}
// 	}
// 	ControlFlow::Continue(())
// }
