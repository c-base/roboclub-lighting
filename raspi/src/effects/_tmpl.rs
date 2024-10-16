use educe::Educe;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::effects::{prelude::*, EffectWindow};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct EffectConfig {
	#[educe(Default = 10.0)]
	something: f32,
}

#[derive(Default)]
pub struct EffectState {}

pub fn effect(config: &EffectConfig, state: &mut EffectState, mut window: EffectWindow) {}
