use educe::Educe;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::effects::{config::color::Color, EffectWindow};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct SolidConfig {
	// #[schema(inline)]
	color: Color,
}

pub fn solid(config: &SolidConfig, _: &mut (), mut window: EffectWindow) {
	for led in window.iter_mut() {
		*led = config.color.value().into();
	}
}
