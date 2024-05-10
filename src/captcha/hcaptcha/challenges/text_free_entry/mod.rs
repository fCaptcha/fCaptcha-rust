use rocket::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use super::{
    Challenge,
    structs::Answer
};

#[derive(Serialize, Deserialize)]
pub struct TextFreeEntryChallenge {
    #[serde(default = "Default::default")]
    pub answers: Vec<Answer<String>>
}

#[async_trait]
impl Challenge<Value> for TextFreeEntryChallenge {
    async fn get_answers(&mut self) -> Value {
        let mut answers = Vec::new();
        for answer in &self.answers {
            answers.push(json!({
                &*answer.task_key: {
                    "text": answer.task_answer
                }
            }));
        }
        return Value::from(answers);
    }

    async fn save_answers_to_database(&mut self) {
        todo!()
    }
}