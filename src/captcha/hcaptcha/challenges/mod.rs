use rocket::async_trait;
mod image_label_binary;
mod area_select;
mod text_free_entry;
mod structs;
mod constants;

#[async_trait]
pub trait Challenge<T> {
    async fn get_answers(&mut self) -> T;
    async fn save_answers_to_database(&mut self);
}
