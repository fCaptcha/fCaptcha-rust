use rocket::async_trait;
use serde_json::Value;
use super::sessions::structs::Task;

pub mod image_label_binary;
pub mod image_label_area_select;
pub mod text_free_entry;
pub mod constants;

#[async_trait]
pub trait Challenge {
    async fn populate_answers_from_database(&mut self, tasks: Vec<&Task>);
    async fn get_answers(&self) -> Value;
    async fn save_answers_to_database(&self);
}
