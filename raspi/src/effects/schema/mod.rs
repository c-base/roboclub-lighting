pub use effect_derive::Schema;

trait Schema {
	fn schema(&self) -> serde_json::Value;
	// fn validate();
}
