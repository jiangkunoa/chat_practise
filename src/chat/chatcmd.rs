use std::{collections::HashMap, sync::Arc};
use anyhow::{anyhow, Ok, Result};

use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{dao::{chatmsg_dao::{create_chat_msg, get_chat_msg, get_chat_msg_limit}, room_dao::{self, get_room, get_rooms_by_member, get_rooms_by_type, update_room_members}, user_dao::get_user_in_id}, models::{chatmsg::ChatMessage, room::Room, user::User}};

use super::chatserver::ChatState;



#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCammand {
    pub cmd: String,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Rooms;

#[repr(i32)]
pub enum RoomType {
    Private = 1,
    Group = 2,
    Public = 3,
}

pub async fn hand_msg(state: Arc<ChatState>, msg: ChatCammand, user: &User) {
    info!("hand msg:{:?}", msg);
    let r = match msg.cmd.as_str() {
        "Rooms" => rooms(state, msg, user).await,
        "CreateRoom" => create_room(state, msg, user).await,
        "Enter" => enter(state, msg, user).await,
        "RoomMsgs" => room_msgs(state, msg, user).await,
        "SendMsg" => send_msg(state, msg, user).await,
        _ => Err(anyhow!(format!("unknown cmd:{:?}", msg))),
    };
    if let Err(e) = r {
        error!("hand msg error:{}", e);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RoomInfo {
    rooms: Vec<Room>,
}

async fn rooms(state: Arc<ChatState>, _msg: ChatCammand, user: &User) -> Result<()> {
    push_rooms(state, user).await
}

#[derive(Debug, Serialize, Deserialize)]
struct ReqCreateRoom {
    room_type: i32,
    room_name: String,
    members: Vec<u64>
}

async fn create_room(state: Arc<ChatState>, msg: ChatCammand, user: &User) -> Result<()> {
    let mut req: ReqCreateRoom = serde_json::from_str(&msg.data)?;
    if !req.members.contains(&user.id) {
        req.members.push(user.id);
    }
    let _ = room_dao::create_room(&state.pool, req.room_type, &req.room_name, &req.members).await?;
    push_rooms(state, user).await
}

#[derive(Debug, Serialize, Deserialize)]
struct ReqEnter {
    room_id: i32
}

async fn enter(state: Arc<ChatState>, msg: ChatCammand, user: &User) -> Result<()> {
    let req: ReqEnter = serde_json::from_str(&msg.data)?;
    let room = get_room(&state.pool, req.room_id).await.ok_or_else(|| anyhow!("room not found"))?;
    let mut members: Vec<u64> = serde_json::from_str(&room.members)?;
    if !members.contains(&user.id) {
        members.push(user.id);
        update_room_members(&state.pool, room.id, members).await?;
    }
    push_rooms(state, user).await?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct ReqRoomMsgs {
    room_id: i32,
    last_id: Option<i32>
}
#[derive(Debug, Serialize, Deserialize)]
struct RspRoomMsgs {
    room_id: i32,
    msgs: Vec<ClientChatMsg>,
}
#[derive(Debug, Serialize, Deserialize)]
struct ClientChatMsg {
    msg: ChatMessage,
    user_name: String
}

async fn room_msgs(state: Arc<ChatState>, msg: ChatCammand, user: &User) -> Result<()> {
    let req: ReqRoomMsgs = serde_json::from_str(&msg.data)?;
    let msgs = match req.last_id {
        Some(last_id) => {
            get_chat_msg_limit(&state.pool, req.room_id, last_id).await?
        },
        None => {
            get_chat_msg(&state.pool, req.room_id).await?
        },
    };
    let ids: Vec<u64> = msgs.iter().map(|msg| msg.sender).collect();
    let users: HashMap<u64, User> = get_user_in_id(&state.pool, &ids).await?.into_iter()
    .map(|user| (user.id, user)) // 使用user.id做key
    .collect();
    let mut chat_msg_list = vec![];
    for chatmsg in &msgs {
        let mybe_user = users.get(&chatmsg.sender);
        if let Some(user) = mybe_user {
            chat_msg_list.push(ClientChatMsg {
                msg: chatmsg.clone(),
                user_name: user.username.clone(),
            });
        } else {
            chat_msg_list.push(ClientChatMsg {
                msg: chatmsg.clone(),
                user_name: "none".to_string(),
            });
        }
    }
    let rsp = ChatCammand {
        cmd: "RspRoomMsgs".to_string(),
        data: serde_json::to_string(&RspRoomMsgs { room_id: req.room_id, msgs: chat_msg_list })?,
    };
    state.conn_map.write().await.get_mut(&user.id).unwrap().send(serde_json::to_string(&rsp)?).await?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct ReqSendMsg {
    room_id: i32,
    msg: String,
}
async fn send_msg(state: Arc<ChatState>, msg: ChatCammand, user: &User) -> Result<()> {
    let req: ReqSendMsg = serde_json::from_str(&msg.data)?;
    let room = get_room(&state.pool, req.room_id).await.ok_or_else(
        || anyhow!("room not found")
    )?;
    let new_msg = create_chat_msg(&state.pool, room.id, &req.msg, user.id).await?;
    let rsp = ChatCammand {
        cmd: "RspSendMsg".to_string(),
        data: serde_json::to_string(&new_msg)?,
    };
    let mut locked = state.conn_map.write().await;
    locked.get_mut(&user.id).unwrap().send(serde_json::to_string(&rsp)?).await?;
    let members: Vec<u64> = serde_json::from_str(&room.members)?;
    members.iter().for_each(|member| {
        if *member != user.id {
            if let Some(sender) = locked.get_mut(member) {
                let _ = sender.send(serde_json::to_string(&rsp).unwrap());
            }
        }
    });
    Ok(())
}

pub async fn push_rooms(state: Arc<ChatState>, user: &User) -> Result<()> {
    let mut rooms = get_rooms_by_member(&state.pool, user.id.to_string().as_str()).await?;
    let mut public_rooms = get_rooms_by_type(&state.pool, RoomType::Public as i32).await?;
    rooms.append(&mut public_rooms);
    let info = RoomInfo { rooms };
    let rsp = ChatCammand {
        cmd: "RspRooms".to_string(),
        data: serde_json::to_string(&info)?,
    };
    state.conn_map.write().await.get_mut(&user.id).unwrap().send(serde_json::to_string(&rsp)?).await?;
    Ok(())
}