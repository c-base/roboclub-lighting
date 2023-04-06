use std::collections::HashMap;

pub use effect_derive::Schema;

// enum SchemaType {}
//
// type SchemaTypeMap = HashMap<String>;

trait Schema {
	fn schema(&self) -> serde_json::Value;
	// fn validate();
}
