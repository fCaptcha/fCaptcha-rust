use redis::aio::ConnectionManager;
use redis::{AsyncCommands, RedisResult};
use rocket::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use structs::Answer;
use crate::captcha::hcaptcha::sessions::{HCaptchaSession, structs::Task};
use crate::captcha_debug;
use super::constants::ANSWERS;
use super::Challenge;

pub mod structs;

pub struct TextFreeEntryChallenge<'a> {
    pub answers: Vec<Answer<String>>,
    pub session: &'a HCaptchaSession
}

#[async_trait]
impl Challenge for TextFreeEntryChallenge<'_> {
    async fn populate_answers_from_database(&mut self, tasks: Vec<&Task>) {
        let mut database = ConnectionManager::clone(ANSWERS.get().await);
        for task in tasks {
            let task_hash = &*task.task_hash;
            let result: RedisResult<String> = database.get(format!("Text Challenges:{0}:{1}", self.session.get_language(), task_hash)).await;
            match result {
                Ok(ref text) => {
                    for mut answer in &mut self.answers {
                        answer.task_answer = String::from(text);
                    }
                }
                Err(error) => {
                    if error.is_io_error() {
                        captcha_debug!("I/O ERORR @ text_free_entry (populate)");
                    }
                }
            }
        }
    }

    async fn get_answers(&self) -> Value {
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

    async fn save_answers_to_database(&self) {
        let language = self.session.get_language();
        for answer in &self.answers {
            let answer_cloned = String::from(&answer.task_answer);
            let hash_cloned = String::from(&answer.task_hash);
            let language_cloned = String::from(&*language);
            let mut database_cloned = ConnectionManager::clone(ANSWERS.get().await);
            tokio::spawn(async move {
                let _result: RedisResult<()> = database_cloned.set(format!("Text Challenges:{0}:{1}", language_cloned, hash_cloned), answer_cloned).await;
            });
        }
    }
}
