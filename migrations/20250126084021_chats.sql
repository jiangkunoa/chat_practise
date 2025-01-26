-- Add migration script here
CREATE TABLE rooms (
    id int PRIMARY KEY AUTO_INCREMENT,
    room_type int NOT NULL,
    room_name VARCHAR(200) NOT NULL,
    members TEXT
);



CREATE TABLE chat_msgs (
    id int PRIMARY KEY AUTO_INCREMENT,
    room_id int NOT NULL,
    message TEXT,
    sender BIGINT,
    send_time VARCHAR(50)
);
