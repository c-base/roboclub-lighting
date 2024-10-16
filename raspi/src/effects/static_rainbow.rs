use educe::Educe;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{color::Hsv, effects::EffectWindow};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct StaticRainbowConfig {
	#[schema(minimum = 0.01, maximum = 1000.0)]
	#[educe(Default = 255.0)]
	hue_frequency: f32,
}

pub fn static_rainbow(config: &StaticRainbowConfig, _: &mut (), mut window: EffectWindow) {
	for (i, led) in window.iter_mut().enumerate() {
		let hue = (i as f32 * (360.0 / config.hue_frequency)) % 360.0;
		*led = Hsv::new(hue, 1.0, 1.0).into();
	}
}
