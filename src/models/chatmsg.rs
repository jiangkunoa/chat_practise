use serde::{Deserialize, Serialize};

 #[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: i32,
    pub room_id: i32,
    pub message: String,
    pub sender: u64,
    pub send_time: String,
}