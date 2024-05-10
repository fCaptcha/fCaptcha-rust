use rocket::serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub datapoint_text: Option<String>,
    pub datapoint_uri: Option<String>,
    pub task_key: String,
}

#[derive(Serialize, Deserialize)]
pub struct CaptchaSession {
    #[serde(rename = "type")]
    pub proof_type: String,
    #[serde(rename = "req")]
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct Root {
    #[serde(rename = "c")]
    pub session_data: CaptchaSession,
    pub key: String,
    pub request_type: String,
    pub requester_question: Value,
    pub requester_question_example: Vec<String>,
    #[serde(rename = "tasklist")]
    pub task_list: Vec<Task>,
}