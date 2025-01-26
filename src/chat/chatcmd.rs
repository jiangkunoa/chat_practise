use std::sync::Arc;
use anyhow::{Result, anyhow};

use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::models::user::User;

use super::chatserver::ChatState;



#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCammand {
    pub cmd: String,
    pub data: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Rooms;

pub fn hand_msg(state: Arc<ChatState>, msg: ChatCammand, user: &User) {
    info!("hand msg:{:?}", msg);
    let r = match msg.cmd.as_str() {
        "Rooms" => rooms(state, msg, user),
        _ => Err(anyhow!(format!("unknown cmd:{:?}", msg))),
    };
    if let Err(e) = r {
        error!("hand msg error:{}", e);
    }
}

fn rooms(state: Arc<ChatState>, msg: ChatCammand, user: &User) -> Result<()> {
    Ok(())
}
