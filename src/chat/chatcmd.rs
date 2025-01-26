use std::sync::Arc;
use anyhow::{Result, anyhow};

use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{dao::{chatmsg_dao::create_chat_msg, room_dao::{get_room, get_rooms_by_member, get_rooms_by_type}}, models::{room::Room, user::User}};

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
    Private,
    Group,
    Public,
}

pub async fn hand_msg(state: Arc<ChatState>, msg: ChatCammand, user: &User) {
    info!("hand msg:{:?}", msg);
    let r = match msg.cmd.as_str() {
        "Rooms" => rooms(state, msg, user).await,
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