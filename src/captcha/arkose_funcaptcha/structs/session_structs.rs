use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionResponse {
	#[serde(rename = "error")]
	pub error: Option<String>,

	#[serde(rename = "token")]
	pub token: Option<String>,

	#[serde(rename = "challenge_url")]
	pub challenge_url: Option<String>,

	#[serde(rename = "challenge_url_cdn")]
	pub challenge_url_cdn: Option<String>,

	#[serde(rename = "noscript")]
	pub noscript: Option<String>,

	#[serde(rename = "mbio")]
	pub mbio: Option<bool>,

	#[serde(rename = "tbio")]
	pub tbio: Option<bool>,

	#[serde(rename = "kbio")]
	pub kbio: Option<bool>,

	#[serde(rename = "disable_default_styling")]
	pub disable_default_styling: Option<bool>,

	#[serde(rename = "string_table")]
	pub string_table: Option<Value>,
}
