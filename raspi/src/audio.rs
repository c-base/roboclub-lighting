use libpulse_binding::{
	callbacks::ListResult,
	context::Context,
	mainloop::standard::{IterateResult, Mainloop},
	operation::{Operation, State},
	proplist::Proplist,
};
use vis_core::{analyzer, recorder::pulse::PulseBuilder, Frames};

use crate::APP_NAME;
use std::{borrow::BorrowMut, cell::RefCell, rc::Rc};

#[derive(Debug, Clone)]
pub struct AnalyzerResult {
	pub analyzer: analyzer::FourierAnalyzer,
	pub average:  analyzer::Spectrum<Vec<f32>>,
	// pub beat_detector: analyzer::BeatDetector,
	pub beat:     usize,
}

fn wait_for_op<T: ?Sized>(mainloop: &mut Mainloop, op: Operation<T>) -> Result<(), String> {
	while let State::Running = op.get_state() {
		match mainloop.iterate(false) {
			IterateResult::Success(_) => {}
			IterateResult::Quit(ret) => {
				return Err(format!(
					"error waiting for pulseaudio operation: quit retval: {}",
					ret.0
				));
			}
			IterateResult::Err(err) => {
				return Err(format!("error waiting for pulseaudio operation: {}", err));
			}
		}
	}

	Ok(())
}

pub fn get_default_monitor() -> Result<String, String> {
	let mut proplist = Proplist::new().unwrap();
	proplist
		.set_str(
			libpulse_binding::proplist::properties::APPLICATION_NAME,
			APP_NAME,
		)
		.unwrap();

	let mut mainloop = Mainloop::new().ok_or("failed creating pulseaudio mainloop")?;
	let mut context = Context::new_with_proplist(&mainloop, APP_NAME, &proplist)
		.ok_or("failed creating pulseaudio context")?;
	context
		.connect(None, libpulse_binding::context::FlagSet::NOFLAGS, None)
		.map_err(|e| format!("failed to connect pulseaudio context: {}", e))?;

	let introspect = context.introspect();

	let mut default_sink: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
	let mut default_sink_ref = default_sink.clone();
	wait_for_op(
		&mut mainloop,
		introspect.get_server_info(move |info| {
			default_sink_ref
				.borrow_mut()
				.replace(info.default_sink_name.clone().map(|s| s.to_string()));
		}),
	)
	.map_err(|e| {
		format!(
			"failed waiting for pulseaudio `get_server_info` operation: {}",
			e
		)
	})?;
	let default_sink = default_sink
		.borrow_mut()
		.replace(None)
		.ok_or("could not get default sink")?;

	let mut default_monitor: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));
	let mut default_monitor_ref = default_monitor.clone();
	wait_for_op(
		&mut mainloop,
		introspect.get_sink_info_by_name(&default_sink, move |info| {
			if let ListResult::Item(item) = info {
				default_monitor_ref
					.borrow_mut()
					.replace(item.monitor_source_name.clone().map(|s| s.to_string()));
			}
		}),
	)
	.map_err(|e| {
		format!(
			"failed waiting for pulseaudio `get_sink_info_by_name` operation: {}",
			e
		)
	})?;
	let default_monitor = default_monitor
		.borrow_mut()
		.replace(None)
		.ok_or("could not get default monitor")?;

	Ok(default_monitor)
}

pub fn get_frames() -> Result<
	Frames<
		AnalyzerResult,
		impl for<'a> FnMut(&'a mut AnalyzerResult, &analyzer::SampleBuffer) -> &'a mut AnalyzerResult,
	>,
	String,
> {
	vis_core::default_config();

	let default_monitor = get_default_monitor()?;

	let analyzer = analyzer::FourierBuilder::new().plan();
	let average = analyzer::Spectrum::new(vec![0.0; analyzer.buckets()], 0.0, 1.0);

	// Beat
	let mut beat = analyzer::BeatBuilder::new().build();
	let mut beat_num = 0;

	let frames = vis_core::Visualizer::new(
		AnalyzerResult {
			analyzer,
			average,
			beat: 0,
		},
		move |info, samples| {
			info.analyzer.analyze(&samples);

			info.average.fill_from(&info.analyzer.average());

			if beat.detect(&samples) {
				beat_num += 1;
			}
			info.beat = beat_num;

			info
		},
	)
	.recorder(
		PulseBuilder::new()
			.device(default_monitor)
			.name(APP_NAME, APP_NAME)
			.build(),
	)
	.async_analyzer(240)
	.frames();

	Ok(frames)
}
