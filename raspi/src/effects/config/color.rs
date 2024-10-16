use educe::Educe;
use palette::Mix;
use rand::Rng;
use serde::{Deserialize, Serialize};
use utoipa::{
	openapi::{KnownFormat, ObjectBuilder, RefOr, SchemaFormat, SchemaType},
	ToSchema,
};

use crate::color::Hsv;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Color {
	#[serde(flatten)]
	value: Hsv,
}

impl Color {
	pub fn value(&self) -> Hsv {
		self.value
	}
}

impl Default for Color {
	fn default() -> Self {
		Color {
			value: Hsv::new(0.0, 0.5, 1.0),
		}
	}
}

impl<'a> ToSchema<'a> for Color {
	fn schema() -> (&'a str, RefOr<utoipa::openapi::Schema>) {
		(
			"Color",
			ObjectBuilder::new()
				.property(
					"hue",
					ObjectBuilder::new()
						.schema_type(SchemaType::Number)
						.format(Some(SchemaFormat::KnownFormat(KnownFormat::Float)))
						.minimum(Some(0.0))
						.maximum(Some(360.0)),
				)
				.required("hue")
				.property(
					"saturation",
					ObjectBuilder::new()
						.schema_type(SchemaType::Number)
						.format(Some(SchemaFormat::KnownFormat(KnownFormat::Float)))
						.minimum(Some(0.0))
						.maximum(Some(1.0)),
				)
				.required("saturation")
				.property(
					"value",
					ObjectBuilder::new()
						.schema_type(SchemaType::Number)
						.format(Some(SchemaFormat::KnownFormat(KnownFormat::Float)))
						.minimum(Some(0.0))
						.maximum(Some(1.0)),
				)
				.required("value")
				.into(),
		)
	}
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, ToSchema)]
pub struct ColorGradient {
	#[schema(inline)]
	from: Color,

	#[schema(inline)]
	to: Color,
}

impl ColorGradient {
	pub fn random(&self) -> Hsv {
		let mut rand = rand::thread_rng();
		let factor = rand.gen::<f32>();
		self.from.value.mix(*self.to.value, factor).into()
	}

	pub fn lerp(&self, factor: f32) -> Hsv {
		self.from.value.mix(*self.to.value, factor).into()
	}
}

impl Default for ColorGradient {
	fn default() -> Self {
		ColorGradient {
			from: Color {
				value: Hsv::new(240.0, 1.0, 1.0),
			},
			to:   Color {
				value: Hsv::new(320.0, 1.0, 1.0),
			},
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum MultiColor {
	Single(Color),
	Multiple(Vec<Color>),
	Gradient(ColorGradient),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Educe, ToSchema)]
#[educe(Default)]
pub struct ColorConfig {
	#[educe(Default = 0.0)]
	hue_min:    f32,
	#[educe(Default = 360.0)]
	hue_max:    f32,
	#[educe(Default = 1.0)]
	brightness: f32,
}

impl ColorConfig {
	pub fn random(&self) -> Hsv {
		let mut rand = rand::thread_rng();
		let hue = if self.hue_max <= self.hue_min {
			self.hue_min
		} else {
			rand.gen_range(self.hue_min..self.hue_max)
		};
		Hsv::new(hue, 1.0, self.brightness)
	}
}
