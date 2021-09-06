use educe::Educe;
use rand::Rng;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::color::HSV;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Educe)]
#[educe(Default)]
pub struct ColorConfig {
	#[educe(Default = 0)]
	hue_min: u8,
	#[educe(Default = 255)]
	hue_max: u8,
}

impl ColorConfig {
	pub fn random(&self) -> HSV {
		let mut rand = rand::thread_rng();
		let hue = if self.hue_max <= self.hue_min {
			self.hue_min
		} else {
			rand.gen_range(self.hue_min..self.hue_max)
		};
		HSV::new(hue, 255, 255)
	}
}
