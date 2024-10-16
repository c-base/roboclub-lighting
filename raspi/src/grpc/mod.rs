pub mod schema;

use std::{
	fmt::Display,
	net::SocketAddr,
	pin::Pin,
	sync::{Arc, Mutex},
};

use eyre::Result;
use futures::{Stream, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use tokio_stream::wrappers::BroadcastStream;
use tonic::{transport::Server, Request, Response, Status};
use tracing::error;

use crate::{
	grpc::schema::{
		controller_server::{Controller, ControllerServer},
		Config,
		DeletePresetRequest,
		DisplayState,
		Effects,
		GroupsResponse,
		LoadPresetRequest,
		Presets,
		SavePresetRequest,
		SegmentsResponse,
		SetGroupsRequest,
		SetPresetRequest,
		SetSegmentsRequest,
		SetStateEffectRequest,
		SetStateRequest,
	},
	runner::{ApiConfig, EffectAPI, EffectRunner},
};

type DisplayStateStream = Pin<Box<dyn Stream<Item = Result<DisplayState, Status>> + Send>>;

pub struct MyController {
	pub(crate) runner: Arc<Mutex<EffectRunner>>,
}

fn wrap_err<E, D>(msg: D) -> impl FnOnce(E) -> Status
where
	E: Into<eyre::Report> + Send + Sync + 'static,
	D: Display + Send + Sync + 'static,
{
	move |err: E| {
		let err = err.into().wrap_err(msg);
		error!("request error: {:?}", err);
		Status::internal(format!("{:#}", err))
	}
}

fn missing_field(field: &str) -> Status {
	Status::invalid_argument(format!("{} is missing (default not accepted)", field))
}

fn transcode<T: DeserializeOwned>(from: &impl Serialize) -> Result<T, Status> {
	let json = serde_json::to_value(from).map_err(wrap_err("serializing to json value"))?;
	let output = serde_json::from_value(json).map_err(wrap_err("deserializing from json value"))?;

	Ok(output)
}

#[tonic::async_trait]
impl Controller for MyController {
	#[tracing::instrument(skip(self))]
	async fn get_config(&self, _: Request<()>) -> Result<Response<Config>, Status> {
		let runner = self.runner.lock().unwrap();
		let cfg = runner
			.get_global_config()
			.map_err(wrap_err("getting global config"))?;

		let reply = Config {
			brightness: cfg.brightness,
			srgb:       cfg.srgb,
		};

		Ok(Response::new(reply))
	}

	#[tracing::instrument(skip(self, request))]
	async fn set_config(&self, request: Request<Config>) -> Result<Response<Config>, Status> {
		let mut runner = self.runner.lock().unwrap();
		let req = request.into_inner();

		let cfg = ApiConfig {
			brightness: req.brightness,
			srgb:       req.srgb,
		};

		runner
			.set_global_config(cfg)
			.map_err(wrap_err("setting global config"))?;

		Ok(Response::new(req))
	}

	#[tracing::instrument(skip(self))]
	async fn list_segments(&self, _: Request<()>) -> Result<Response<SegmentsResponse>, Status> {
		let runner = self.runner.lock().unwrap();
		let strips = runner
			.list_segments()
			.map_err(wrap_err("getting segments"))?;

		let mut proto_strips = Vec::with_capacity(strips.len());
		for strip in strips {
			proto_strips.push(strip.try_into()?);
		}

		let reply = SegmentsResponse {
			strips: proto_strips,
		};

		Ok(Response::new(reply))
	}

	#[tracing::instrument(skip(self, request))]
	async fn set_segments(
		&self,
		request: Request<SetSegmentsRequest>,
	) -> Result<Response<SegmentsResponse>, Status> {
		let mut runner = self.runner.lock().unwrap();

		let strips = request.into_inner().strips;

		let mut config_strips = Vec::with_capacity(strips.len());
		for strip in strips.clone() {
			config_strips.push(strip.try_into()?);
		}

		runner
			.set_segments(config_strips)
			.map_err(wrap_err("setting segments"))?;

		let reply = SegmentsResponse { strips };

		Ok(Response::new(reply))
	}

	async fn list_groups(&self, _: Request<()>) -> Result<Response<GroupsResponse>, Status> {
		let runner = self.runner.lock().unwrap();

		let groups = runner.list_groups().map_err(wrap_err("getting groups"))?;

		let mut proto_groups = Vec::with_capacity(groups.len());
		for group in groups {
			proto_groups.push(group.try_into()?);
		}

		let reply = GroupsResponse {
			groups: proto_groups,
		};

		Ok(Response::new(reply))
	}

	#[tracing::instrument(skip(self, request))]
	async fn set_groups(
		&self,
		request: Request<SetGroupsRequest>,
	) -> Result<Response<GroupsResponse>, Status> {
		let mut runner = self.runner.lock().unwrap();
		let groups = request.into_inner().groups;

		let mut config_groups = Vec::with_capacity(groups.len());
		for group in groups.clone() {
			config_groups.push(group.try_into()?);
		}

		runner
			.set_groups(config_groups)
			.map_err(wrap_err("setting groups"))?;

		let reply = GroupsResponse { groups };

		Ok(Response::new(reply))
	}

	#[tracing::instrument(skip(self))]
	async fn list_effects(&self, _: Request<()>) -> Result<Response<Effects>, Status> {
		let runner = self.runner.lock().unwrap();
		let effects = runner
			.list_effects()
			.map_err(wrap_err("getting global config"))?;

		let reply = effects.try_into()?;

		Ok(Response::new(reply))
	}

	#[tracing::instrument(skip(self))]
	async fn list_presets(&self, _: Request<()>) -> Result<Response<Presets>, Status> {
		let runner = self.runner.lock().unwrap();

		let presets = runner
			.list_presets()
			.map_err(wrap_err("setting global config"))?;

		let reply = presets.clone().try_into()?;

		Ok(Response::new(reply))
	}

	#[tracing::instrument(skip(self, request))]
	async fn set_preset(
		&self,
		request: Request<SetPresetRequest>,
	) -> Result<Response<DisplayState>, Status> {
		let mut runner = self.runner.lock().unwrap();

		let SetPresetRequest { name, data } = request.into_inner();
		let data = data.ok_or(missing_field("SetPresetRequest.data"))?;

		runner
			.set_preset(name, data.clone().try_into()?)
			.map_err(wrap_err("setting preset"))?;

		Ok(Response::new(data))
	}

	#[tracing::instrument(skip(self, request))]
	async fn delete_preset(
		&self,
		request: Request<DeletePresetRequest>,
	) -> Result<Response<()>, Status> {
		let mut runner = self.runner.lock().unwrap();

		let DeletePresetRequest { name } = request.into_inner();

		runner
			.delete_preset(name)
			.map_err(wrap_err("setting preset"))?;

		Ok(Response::new(()))
	}

	#[tracing::instrument(skip(self, request))]
	async fn load_preset(
		&self,
		request: Request<LoadPresetRequest>,
	) -> Result<Response<DisplayState>, Status> {
		let mut runner = self.runner.lock().unwrap();
		runner
			.load_preset(request.into_inner().name)
			.map_err(wrap_err("loading preset"))?;

		let state = runner
			.get_state()
			.map_err(wrap_err("getting state"))?
			.clone();

		Ok(Response::new(state.try_into()?))
	}

	#[tracing::instrument(skip(self, request))]
	async fn save_preset(
		&self,
		request: Request<SavePresetRequest>,
	) -> Result<Response<DisplayState>, Status> {
		let mut runner = self.runner.lock().unwrap();
		runner
			.save_preset(request.into_inner().name)
			.map_err(wrap_err("saving preset"))?;

		let state = runner
			.get_state()
			.map_err(wrap_err("getting state"))?
			.clone();

		Ok(Response::new(state.try_into()?))
	}

	#[tracing::instrument(skip(self))]
	async fn get_state(&self, _: Request<()>) -> Result<Response<DisplayState>, Status> {
		let runner = self.runner.lock().unwrap();
		let state = runner
			.get_state()
			.map_err(wrap_err("getting state"))?
			.clone();

		Ok(Response::new(state.try_into()?))
	}

	#[tracing::instrument(skip(self, request))]
	async fn set_state(
		&self,
		request: Request<SetStateRequest>,
	) -> Result<Response<DisplayState>, Status> {
		let mut runner = self.runner.lock().unwrap();

		// protobuf kinda stupid, this really should not be optional.
		let new_state = request
			.into_inner()
			.state
			.ok_or(missing_field("SetStateRequest.state"))?;

		let state = new_state.clone().try_into()?;
		// println!("{:#?}", state);

		runner.set_state(state).map_err(wrap_err("setting state"))?;

		Ok(Response::new(new_state))
	}

	#[tracing::instrument(skip(self, request))]
	async fn set_state_effect(
		&self,
		request: Request<SetStateEffectRequest>,
	) -> Result<Response<DisplayState>, Status> {
		let mut runner = self.runner.lock().unwrap();
		let mut state = runner
			.get_state()
			.map_err(wrap_err("getting state"))?
			.clone();

		let SetStateEffectRequest { index, effect } = request.into_inner();

		let index: usize = index
			.try_into()
			.map_err(wrap_err("converting SetStateEffectRequest.index"))?;

		if let Some(state_effect) = state.effects.get_mut(index) {
			*state_effect = effect
				.ok_or(missing_field("SetStateEffectRequest.effect"))?
				.try_into()?;
		} else {
			return Err(Status::invalid_argument(
				"SetStateEffectRequest.index is not in the current state.",
			));
		};

		runner
			.set_state(state.clone())
			.map_err(wrap_err("setting state"))?;

		Ok(Response::new(state.try_into()?))
	}

	type StreamStateStream = DisplayStateStream;

	#[tracing::instrument(skip(self))]
	async fn stream_state(
		&self,
		_: Request<()>,
	) -> Result<Response<Self::StreamStateStream>, Status> {
		let runner = self.runner.lock().unwrap();
		let rx = runner.subscribe();

		Ok(Response::new(Box::pin(BroadcastStream::new(rx).map(
			|res| {
				let state = res.map_err(|err| Status::deadline_exceeded(err.to_string()))?;
				let state: DisplayState = state.try_into()?;

				Ok(state)
			},
		))))
	}
}

pub async fn run(runner: Arc<Mutex<EffectRunner>>) -> Result<()> {
	// let addr = "[::1]:4445".parse()?;
	let addr = SocketAddr::from(([0, 0, 0, 0], 4445));
	tracing::debug!("grpc listening on {}", addr);

	let controller = MyController { runner };
	let controller = ControllerServer::new(controller);

	Server::builder()
		.accept_http1(true)
		.add_service(tonic_web::enable(controller))
		.serve(addr)
		.await?;

	Ok(())
}
