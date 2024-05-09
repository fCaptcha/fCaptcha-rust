use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomGUI {
	#[serde(rename = "_challenge_imgs")]
	pub _challenge_imgs: Option<Vec<String>>,
	#[serde(rename = "audio_challenge_urls")]
	pub audio_challenge_urls: Option<Vec<String>>,
	// funcap tends to sometimes hide variant here.
	#[serde(rename = "instruction_string")]
	pub instruction_string: Option<String>
}
#[derive(Serialize, Deserialize, Debug)]
pub struct GameData {
	#[serde(rename = "display_fc_welldone")]
	pub display_fc_welldone: Option<bool>,
	#[serde(rename = "final_challenge_text")]
	pub final_challenge_text: Option<String>,
	#[serde(rename = "customGUI")]
	pub custom_gui: Option<CustomGUI>,
	#[serde(rename = "instruction_string")]
	pub instruction_string: Option<String>,
	#[serde(rename = "game_variant")]
	pub game_variant: Option<String>,
	#[serde(rename = "game_difficulty")]
	pub game_difficulty: Option<i32>,
	#[serde(rename = "gameType")]
	pub game_type: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Game {
	#[serde(rename = "error")]
	pub error: Option<String>,
	#[serde(rename = "session_token")]
	pub session_token: Option<String>,
	#[serde(rename = "challengeID")]
	pub challenge_id: Option<String>,
	#[serde(rename = "challengeURL")]
	pub challenge_u_r_l: Option<String>,
	#[serde(rename = "game_data")]
	pub game_data: Option<GameData>,
	#[serde(rename = "game_sid")]
	pub game_sid: Option<String>,
	#[serde(rename = "sid")]
	pub sid: Option<String>,
	#[serde(rename = "lang")]
	pub lang: Option<String>,
	#[serde(rename = "style_theme")]
	pub style_theme: Option<String>,
	#[serde(rename = "dapib_url")]
	pub dapi_breakers: Option<String>,
	pub audio_challenge_urls: Option<Vec<String>>
}


#[derive(Serialize, Deserialize, Debug)]
pub struct AnswerResponse {
	#[serde(rename = "response")]
	pub response: Option<String>,
	#[serde(rename = "solved")]
	pub solved: Option<bool>,
	#[serde(rename = "decryption_key")]
	pub decryption_key: Option<String>,
	#[serde(rename = "incorrect_guess")]
	pub incorrect_guess: Option<String>
}
