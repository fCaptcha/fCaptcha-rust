use rocket::async_trait;
mod image_label_binary;
mod image_label_area_select;
mod text_free_entry;
mod constants;

#[async_trait]
pub trait Challenge<T> {

    async fn populate_answers_from_database(&mut self);
    async fn get_answers(&self) -> T;
    async fn save_answers_to_database(&self);
}
