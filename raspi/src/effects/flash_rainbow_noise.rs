use std::time::Duration;

use educe::Educe;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
	effects::{config::color::ColorGradient, prelude::*, EffectWindow},
	noise,
};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct FlashRainbowNoiseConfig {
	// #[schema(inline)]
	colors: ColorGradient,

	#[schema(minimum = 0.00001, maximum = 5.0)]
	#[educe(Default = 0.15)]
	period: f32,

	#[schema(minimum = 0.00001, maximum = 0.99999)]
	#[educe(Default = 0.001)]
	on_percentage: f32,

	#[schema(minimum = 0.00001, maximum = 20.0)]
	#[educe(Default = 0.03)]
	speed: f32,

	#[schema(minimum = 0.00001, maximum = 500.0)]
	#[educe(Default = 20.0)]
	size: f32,

	#[schema(minimum = 0.0, maximum = 1.0)]
	#[educe(Default = 0.05)]
	threshold: f32,
}

#[derive(Default)]
pub struct FlashRainbowNoiseState {
	timer:   TimerState,
	counter: f32,
}

pub fn flash_rainbow_noise(
	config: &FlashRainbowNoiseConfig,
	state: &mut FlashRainbowNoiseState,
	mut window: EffectWindow,
) {
	let period = Duration::from_secs_f32(config.period);
	let t = state.timer.tick(period);

	state.counter += config.speed;

	if !t.triggered {
		if t.percentage > config.on_percentage {
			clear_all_raw(&mut window);
		}

		return;
	}

	let color = config.colors.random();

	// let data = ctrl.state_mut();
	// for (strip_num, strip) in data.iter_mut().enumerate() {
	for (led_num, led) in window.iter_mut().enumerate() {
		let noise_val = noise::simplex3d(
			led_num as f32 / config.size,
			/* strip_num as f32 * */ 100.0,
			state.counter,
		);
		if noise_val > config.threshold {
			*led = color.into();
		}
	}
	// }

	// ctrl.write_state();
	// sleep_ms((config.period * config.on_percentage * 1000.0) as u64);
	//
	// set_all(ctrl, &Rgba::default());
	// let now = std::time::Instant::now();
	// let diff = now - start;
	// sleep_ms((config.period * 1000.0) as u64 - diff.as_millis() as u64);
}
