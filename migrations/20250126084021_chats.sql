-- Add migration script here
CREATE TABLE rooms (
    id int PRIMARY KEY AUTO_INCREMENT,
    room_type int NOT NULL,
    room_name VARCHAR(200) NOT NULL,
    members TEXT
);