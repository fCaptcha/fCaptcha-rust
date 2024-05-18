pub mod structs;

use std::str::FromStr;
use redis::{
    aio::ConnectionManager,
    AsyncCommands,
    RedisResult
};
use super::{
    super::sessions::{
        HCaptchaSession,
        structs::Task
    },
    constants::ANSWERS,
    Challenge
};
use rocket::async_trait;
use serde_json::{json, Value};
use self::structs::Answer;

pub struct ImageLabelAreaSelect<'a> {
    pub answers: Vec<Answer<Value>>,
    session: &'a HCaptchaSession
}

#[async_trait]
impl Challenge for ImageLabelAreaSelect<'_> {
    async fn populate_answers_from_database(&mut self, tasks: Vec<&Task>) {
        let mut database = ConnectionManager::clone(ANSWERS.get().await);
        for task in tasks {
            let task_hash = &*task.task_hash;
            let result: RedisResult<String> = AsyncCommands::get(&mut database, format!("Image Challenges:{0}:{1}", self.session.get_requester_question().await, task_hash)).await;
            match result {
                Ok(ref text) => {
                    for mut answer in &mut self.answers {
                        if let Ok(value) = Value::from_str(text) {
                            answer.task_answer = value;
                        }
                    }
                }
                Err(_) => {}
            }
        }
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
        for answer in &self.answers {
            let answer_cloned = Value::clone(&answer.task_answer);
            let hash_cloned = String::from(&answer.task_hash);
            let mut database_cloned = ConnectionManager::clone(ANSWERS.get().await);
            tokio::spawn(async move {
                let _result: RedisResult<()> = database_cloned.set(format!("Image Challenges:{0}", hash_cloned), answer_cloned.to_string()).await;
            });
        }
    }
}