use serde::{Deserialize, Serialize};


#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct Room {
    pub id: i32,
    pub room_type: i32,
    pub room_name: String,
    pub members: String,
}
