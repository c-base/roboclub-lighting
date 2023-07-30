use std::ops::Deref;

use educe::Educe;
use effect_derive::Schema;
use palette::Mix;
use rand::Rng;
use schemars::{
	gen::SchemaGenerator,
	schema::{InstanceType, Schema, SchemaObject},
	JsonSchema,
};
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
		self.value.clone()
	}
}

impl Default for Color {
	fn default() -> Self {
		Color {
			value: Hsv::new(0.0, 0.3, 1.0),
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

impl JsonSchema for Color {
	fn schema_name() -> String {
		"Color".to_string()
	}

	fn json_schema(gen: &mut SchemaGenerator) -> Schema {
		let mut schema_object = SchemaObject {
			instance_type: Some(InstanceType::Object.into()),
			..Default::default()
		};
		let object = schema_object.object();

		object.properties.insert(
			"hue".into(),
			Schema::Object({
				let mut schema_object = SchemaObject {
					instance_type: Some(InstanceType::Number.into()),
					..Default::default()
				};
				let num = schema_object.number();
				num.minimum = Some(0.0);
				num.maximum = Some(360.0);

				schema_object
			}),
		);
		object.required.insert("hue".into());

		object.properties.insert(
			"saturation".into(),
			Schema::Object({
				let mut schema_object = SchemaObject {
					instance_type: Some(InstanceType::Number.into()),
					..Default::default()
				};
				let num = schema_object.number();
				num.minimum = Some(0.0);
				num.maximum = Some(1.0);

				schema_object
			}),
		);
		object.required.insert("saturation".into());

		object.properties.insert(
			"value".into(),
			Schema::Object({
				let mut schema_object = SchemaObject {
					instance_type: Some(InstanceType::Number.into()),
					..Default::default()
				};
				let num = schema_object.number();
				num.minimum = Some(0.0);
				num.maximum = Some(1.0);

				schema_object
			}),
		);
		object.required.insert("value".into());

		Schema::Object(schema_object)
	}
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
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
		self.from.value.mix(&self.to.value, factor).into()
	}

	pub fn lerp(&self, factor: f32) -> Hsv {
		self.from.value.mix(&self.to.value, factor).into()
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema)]
#[serde(tag = "type")]
pub enum MultiColor {
	Single(Color),
	Multiple(Vec<Color>),
	Gradient(ColorGradient),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Educe, ToSchema)]
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
