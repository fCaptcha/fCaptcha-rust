mod structs;

use rocket::async_trait;
use serde_json::{json, Value};
use super::Challenge;
use self::structs::Answer;

pub struct ImageLabelAreaSelect {
    pub answers: Vec<Answer<Vec<Value>>>
}

#[async_trait]
impl Challenge<Value> for ImageLabelAreaSelect {
    async fn populate_answers_from_database(&mut self) {
        todo!()
    }

    async fn get_answers(&self) -> Value {
        let mut answers = Vec::new();
        for answer in &self.answers {
            answers.push(json!({
                &*answer.task_key: answer.task_answer
            }));
        }
        return Value::from(answers);
    }

    async fn save_answers_to_database(&self) {
        todo!()
    }
}