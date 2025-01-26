/**
 * 根据这个表结构和room.rs编写常规dao方法
 * CREATE TABLE chat_msgs (
    id int PRIMARY KEY AUTO_INCREMENT,
    room_id int NOT NULL,
    message TEXT,
    sender BIGINT,
    send_time VARCHAR(50)
);
 */

 use crate::models::chatmsg::ChatMessage;
 use sqlx::{types::chrono, MySql, Pool};
 
 pub async fn get_chat_msg(pool: &Pool<MySql>, room_id: i32) -> Result<Vec<ChatMessage>, sqlx::Error> {
     sqlx::query_as::<_, ChatMessage>("SELECT * FROM chat_msgs WHERE room_id = ?")
         .bind(room_id)
         .fetch_all(pool)
         .await
 }

 pub async fn create_chat_msg(pool: &Pool<MySql>, room_id: i32, message: &str, sender: u64) -> Result<ChatMessage, sqlx::Error> {
     sqlx::query_as::<_, ChatMessage>("INSERT INTO chat_msgs (room_id, message, sender, send_time) VALUES (?, ?, ?, ?)")
         .bind(room_id)
         .bind(message)
         .bind(sender)
         .bind(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
         .fetch_one(pool)
         .await
 }

