trait Schema {
	fn schema(&self) -> serde_json::Value;
}
