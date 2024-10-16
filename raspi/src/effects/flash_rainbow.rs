use std::time::Duration;

use educe::Educe;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::effects::{config::color::ColorGradient, prelude::*, EffectWindow};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct FlashRainbowConfig {
	// #[schema(inline)]
	colors: ColorGradient,

	#[schema(minimum = 0.00001, maximum = 5.0)]
	#[educe(Default = 0.2)]
	period: f32,

	#[schema(minimum = 0.00001, maximum = 0.99999)]
	#[educe(Default = 0.1)]
	on_percentage: f32,
}

#[derive(Default)]
pub struct FlashRainbowState {
	timer: TimerState,
}

pub fn flash_rainbow(
	config: &FlashRainbowConfig,
	state: &mut FlashRainbowState,
	mut window: EffectWindow,
) {
	let period = Duration::from_secs_f32(config.period);
	let t = state.timer.tick(period);

	if t.triggered {
		let color = config.colors.random();
		set_all_raw(&mut window, color.into());
	} else if t.percentage > config.on_percentage {
		clear_all_raw(&mut window);
	}
}
