use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Triggers {
	pub events: HashMap<String, String>,
	pub messages: HashMap<String, String>,
}

impl Triggers {
	pub fn new() -> Self {
		return Self {
			events: HashMap::new(),
			messages: HashMap::new(),
		}
	}
}