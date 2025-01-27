-- Add migration script here
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    avatar TEXT
);

CREATE TABLE rooms (
    id int PRIMARY KEY AUTO_INCREMENT,
    room_type int NOT NULL,
    room_name VARCHAR(200) NOT NULL,
    members TEXT
);


drop table if exists chat_msgs;
CREATE TABLE chat_msgs (
    id int PRIMARY KEY AUTO_INCREMENT,
    room_id int NOT NULL,
    message TEXT,
    sender BIGINT,
    send_time VARCHAR(50)
);


insert into rooms (room_type, room_name, members) values (3, '公共', '[]');