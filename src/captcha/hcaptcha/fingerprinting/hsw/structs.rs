use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(super) struct EncDecAPIResponse {
    data: String
}