use std::collections::{HashMap, HashSet};

use tonic::Status;

pub use crate::grpc::schema::generated::*;
use crate::{
	config,
	effects,
	grpc::{missing_field, transcode, wrap_err},
};

mod generated {
	tonic::include_proto!("lighting"); // The string specified here must match the proto package name
}

impl TryFrom<config::DisplayState> for DisplayState {
	type Error = Status;

	fn try_from(value: config::DisplayState) -> Result<Self, Self::Error> {
		let mut effects = Vec::with_capacity(value.effects.len());
		for effect in value.effects {
			effects.push(effect.try_into()?);
		}

		Ok(DisplayState { effects })
	}
}

impl TryFrom<DisplayState> for config::DisplayState {
	type Error = Status;

	fn try_from(value: DisplayState) -> Result<Self, Self::Error> {
		let mut effects = Vec::with_capacity(value.effects.len());
		for effect in value.effects {
			effects.push(effect.try_into()?);
		}

		Ok(config::DisplayState { effects })
	}
}

impl TryFrom<config::DisplayStateEffect> for DisplayStateEffect {
	type Error = Status;

	fn try_from(value: config::DisplayStateEffect) -> Result<Self, Self::Error> {
		let mut segment_ids = Vec::with_capacity(value.segment_ids.len());
		for segment_id in value.segment_ids {
			segment_ids.push(segment_id.try_into()?);
		}

		Ok(DisplayStateEffect {
			effect_id: value.effect_id,
			config: Some(
				serde_json::from_value(value.config)
					.map_err(wrap_err("converting to grpc struct"))?,
			),
			segment_ids,
			group_ids: value.group_ids.into_iter().collect(),
		})
	}
}

impl TryFrom<DisplayStateEffect> for config::DisplayStateEffect {
	type Error = Status;

	fn try_from(value: DisplayStateEffect) -> Result<Self, Self::Error> {
		let mut segment_ids = HashSet::with_capacity(value.segment_ids.len());
		for segment_id in value.segment_ids {
			segment_ids.insert(segment_id.try_into()?);
		}

		Ok(config::DisplayStateEffect {
			effect_id: value.effect_id,
			config: serde_json::to_value(
				value
					.config
					.ok_or(missing_field("DisplayStateEffect.config"))?,
			)
			.map_err(wrap_err("converting from grpc struct"))?,
			segment_ids,
			group_ids: value.group_ids.into_iter().collect(),
		})
	}
}

impl TryFrom<config::SegmentId> for SegmentId {
	type Error = Status;

	fn try_from(value: config::SegmentId) -> Result<Self, Self::Error> {
		Ok(SegmentId {
			strip:   value
				.strip_idx
				.try_into()
				.map_err(wrap_err("converting SegmentId.strip"))?,
			segment: value
				.segment_idx
				.try_into()
				.map_err(wrap_err("converting SegmentId.segment"))?,
		})
	}
}

impl TryFrom<SegmentId> for config::SegmentId {
	type Error = Status;

	fn try_from(value: SegmentId) -> Result<Self, Self::Error> {
		Ok(config::SegmentId {
			strip_idx:   value
				.strip
				.try_into()
				.map_err(wrap_err("converting SegmentId.strip"))?,
			segment_idx: value
				.segment
				.try_into()
				.map_err(wrap_err("converting SegmentId.segment"))?,
		})
	}
}

impl TryFrom<HashMap<String, effects::EffectData>> for Effects {
	type Error = Status;

	fn try_from(value: HashMap<String, effects::EffectData>) -> Result<Self, Self::Error> {
		let mut effects = HashMap::with_capacity(value.len());
		for (name, effect) in value {
			effects.insert(name, effect.try_into()?);
		}

		Ok(Effects { effects })
	}
}

impl TryFrom<Effects> for HashMap<String, effects::EffectData> {
	type Error = Status;

	fn try_from(value: Effects) -> Result<Self, Self::Error> {
		let mut effects = HashMap::with_capacity(value.effects.len());
		for (name, effect) in value.effects {
			effects.insert(name, effect.try_into()?);
		}

		Ok(effects)
	}
}

impl TryFrom<effects::EffectData> for Effect {
	type Error = Status;

	fn try_from(value: effects::EffectData) -> Result<Self, Self::Error> {
		Ok(Effect {
			id:             value.id,
			name:           value.name,
			schema:         Some(transcode(&value.schema)?),
			default_config: Some(
				serde_json::from_value(value.default_config)
					.map_err(wrap_err("deserializing default config"))?,
			),
		})
	}
}

impl TryFrom<Effect> for effects::EffectData {
	type Error = Status;

	fn try_from(value: Effect) -> Result<Self, Self::Error> {
		Ok(effects::EffectData {
			id:             value.id,
			name:           value.name,
			schema:         transcode(&value.schema.ok_or(missing_field("Effect.schema"))?)?,
			default_config: serde_json::to_value(
				value
					.default_config
					.ok_or(missing_field("Effect.default_config"))?,
			)
			.map_err(wrap_err("serializing default config"))?,
		})
	}
}

// impl TryFrom<Vec<config::Strip>> for Segments {
// 	type Error = Status;
//
// 	fn try_from(value: Vec<config::Strip>) -> Result<Self, Self::Error> {
// 		let mut strips = Vec::with_capacity(value.len());
// 		for strip in value {
// 			strips.push(strip.try_into()?);
// 		}
//
// 		Ok(Segments { strips })
// 	}
// }
//
// impl TryFrom<Segments> for Vec<config::Strip> {
// 	type Error = Status;
//
// 	fn try_from(value: Segments) -> Result<Self, Self::Error> {
// 		let mut strips = Vec::with_capacity(value.strips.len());
// 		for strip in value.strips {
// 			strips.push(strip.try_into()?);
// 		}
//
// 		Ok(strips)
// 	}
// }

impl TryFrom<config::Strip> for Strip {
	type Error = Status;

	fn try_from(value: config::Strip) -> Result<Self, Self::Error> {
		let mut segments = Vec::with_capacity(value.segments.len());
		for segment in value.segments {
			segments.push(segment.try_into()?);
		}

		Ok(Strip {
			offset: value.offset as u32,
			segments,
		})
	}
}

impl TryFrom<Strip> for config::Strip {
	type Error = Status;

	fn try_from(value: Strip) -> Result<Self, Self::Error> {
		let mut segments = Vec::with_capacity(value.segments.len());
		for segment in value.segments {
			segments.push(segment.try_into()?);
		}

		Ok(config::Strip {
			offset: value.offset as usize,
			segments,
		})
	}
}

impl TryFrom<config::Segment> for Segment {
	type Error = Status;

	fn try_from(value: config::Segment) -> Result<Self, Self::Error> {
		Ok(Segment {
			name:     value.name,
			length:   value
				.length
				.try_into()
				.map_err(wrap_err("converting Segment.length"))?,
			reversed: value.reversed,
		})
	}
}

impl TryFrom<Segment> for config::Segment {
	type Error = Status;

	fn try_from(value: Segment) -> Result<Self, Self::Error> {
		Ok(config::Segment {
			name:     value.name,
			length:   value
				.length
				.try_into()
				.map_err(wrap_err("converting Segment.length"))?,
			reversed: value.reversed,
		})
	}
}

impl TryFrom<config::Group> for Group {
	type Error = Status;

	fn try_from(value: config::Group) -> Result<Self, Self::Error> {
		let mut segment_ids = Vec::with_capacity(value.segment_ids.len());
		for segment in value.segment_ids {
			segment_ids.push(segment.try_into()?);
		}

		Ok(Group {
			id: value.id,
			name: value.name,
			segment_ids,
		})
	}
}

impl TryFrom<Group> for config::Group {
	type Error = Status;

	fn try_from(value: Group) -> Result<Self, Self::Error> {
		let mut segment_ids = HashSet::with_capacity(value.segment_ids.len());
		for segment in value.segment_ids {
			segment_ids.insert(segment.try_into()?);
		}

		Ok(config::Group {
			id: value.id,
			name: value.name,
			segment_ids,
		})
	}
}

impl TryFrom<HashMap<String, config::DisplayState>> for Presets {
	type Error = Status;

	fn try_from(value: HashMap<String, config::DisplayState>) -> Result<Self, Self::Error> {
		let mut presets = HashMap::with_capacity(value.len());
		for (name, state) in value {
			presets.insert(name, state.try_into()?);
		}

		Ok(Presets { presets })
	}
}

impl TryFrom<Presets> for HashMap<String, config::DisplayState> {
	type Error = Status;

	fn try_from(value: Presets) -> Result<Self, Self::Error> {
		let mut presets = HashMap::with_capacity(value.presets.len());
		for (name, state) in value.presets {
			presets.insert(name, state.try_into()?);
		}

		Ok(presets)
	}
}
