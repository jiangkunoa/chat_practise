use std::{collections::HashMap, net::SocketAddr, sync::{Arc, RwLock}};

use anyhow::Result;
use futures::{SinkExt, StreamExt};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, Pool};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

use crate::{dao::user_dao, web::jwt};

type ConnSender = tokio::sync::mpsc::Sender<String>;
type ConnMap = Arc<RwLock<HashMap<u64, ConnSender>>>;

#[derive(Debug)]
pub struct ChatState {
    pub conn_map: ConnMap,
    pub pool: Pool<MySql>,
}

pub async fn start_chat_server(port: u16, pool: Pool<MySql>) -> Result<Arc<ChatState>> {
    let state = ChatState {
        conn_map: Arc::new(RwLock::new(HashMap::new())),
        pool: pool,
    };
    let state = Arc::new(state);
    let result = state.clone();
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    tokio::spawn(async move {
        info!("start chat server on {}", port);
        while let Ok((stream, addr)) = listener.accept().await {
            let state = state.clone();
            tokio::spawn(async move {
                match hand_connect(stream, state, addr).await {
                    Ok(_) => {
                        info!("hand finish {}", addr);
                    }
                    Err(e) => {
                        info!("hand connect error:{}", e);
                    }
                }
            });
        }
    });
    Ok(result)
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthUser {
    id: u64,
    username: String,
}

async fn hand_connect(stream: TcpStream, state: Arc<ChatState>, addr: SocketAddr) -> Result<()> {  
    info!("new connect from {}", addr);
    let (read, write) = stream.into_split();
    let mut framed = FramedRead::new(read, LengthDelimitedCodec::new());
    //auth user
    let user;
    if let Some(data) = framed.next().await {
        info!("auth user:{}", addr);
        let auth_msg = String::from_utf8(data?.to_vec())?;
        info!("auth_msg {}", auth_msg);
        if auth_msg.is_empty() {
            info!("auth user error1 {}", addr);
            return Err(anyhow::anyhow!("auth user error"));
        }
        let claims = jwt::validate_jwt(auth_msg.as_str())?;
        user = user_dao::get_user(&state.pool, claims.sub).await.ok_or_else(|| anyhow::anyhow!("user not found"))?;
    } else {
        info!("auth user error2 {}", addr);
        return Err(anyhow::anyhow!("read line from stream error"));
    }
    info!("auth user success:{}, {}", user.username, addr);
    let (sender, receiver) = tokio::sync::mpsc::channel::<String>(10);
    state.conn_map.write().expect("system lock error")
        .insert(user.id, sender);

    tokio::spawn(async move {
        let mut receiver = receiver;
        let mut frame_writer = FramedWrite::new(write, LengthDelimitedCodec::new());
        while let Some(msg) = receiver.recv().await {
            match frame_writer.send(bytes::Bytes::from(msg)).await {
                Ok(_) => {}
                Err(e) => {
                    info!("send msg error:{}", e);
                }
            }
        }
    });
    loop {
        if let Some(data) = framed.next().await {
            let logic_msg = String::from_utf8(data?.to_vec())?;
            info!("recv msg:{}", logic_msg);
        } else {
            info!("close connect:{}, {}", user.username, addr);
            state.conn_map.write().expect("system lock error").remove(&user.id);
            return Err(anyhow::anyhow!("read line from stream error"));
        }
    }
}

