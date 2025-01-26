use anyhow::Result;
use sqlx::MySqlPool;

use crate::models::room::Room;


pub async fn get_room(pool: &MySqlPool, id: i32) -> Option<Room> {
    sqlx::query_as::<_, Room>("SELECT * FROM rooms WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .ok()
}

pub async fn get_room_by_name(pool: &MySqlPool, room_name: &str) -> Option<Room> {
    sqlx::query_as::<_, Room>("SELECT * FROM rooms WHERE room_name = ?")
        .bind(room_name)
        .fetch_one(pool)
        .await
        .ok()
}

pub async fn create_room(pool: &MySqlPool, room_type: i32, room_name: &str, members: Vec<u64>) -> Result<Room> {
    let members = serde_json::to_string(&members)?;
    sqlx::query_as::<_, Room>("INSERT INTO rooms (room_type, room_name, members) VALUES (?, ?, ?)")
        .bind(room_type)
        .bind(room_name)
        .bind(members)
        .fetch_one(pool)
        .await
        .map_err(|e| e.into())
}

pub async fn update_room_members(pool: &MySqlPool, id: i32, members: Vec<u64>) -> Result<()> {
    let members = serde_json::to_string(&members)?;
    sqlx::query("UPDATE rooms SET members = ? WHERE id = ?")
        .bind(members)
        .bind(id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|e| e.into())
}

pub async fn delete_room(pool: &MySqlPool, id: i32) -> Result<()> {
    sqlx::query("DELETE FROM rooms WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map(|_| ())
        .map_err(|e| e.into())
}

pub async fn get_rooms(pool: &MySqlPool) -> Result<Vec<Room>> {
    sqlx::query_as::<_, Room>("SELECT * FROM rooms")
        .fetch_all(pool)
        .await
        .map_err(|e| e.into())
}

pub async fn get_rooms_by_member(pool: &MySqlPool, member: &str) -> Result<Vec<Room>> {
    sqlx::query_as::<_, Room>("SELECT * FROM rooms WHERE members LIKE ?")
        .bind(format!("%{}%", member))
        .fetch_all(pool)
        .await
        .map_err(|e| e.into())
}

pub async fn get_rooms_by_type(pool: &MySqlPool, room_type: i32) -> Result<Vec<Room>> {
    sqlx::query_as::<_, Room>("SELECT * FROM rooms WHERE room_type = ?")
        .bind(room_type)
        .fetch_all(pool)
        .await
        .map_err(|e| e.into())
}