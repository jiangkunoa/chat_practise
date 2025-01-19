use std::env;

use anyhow::Result;
use futures::{SinkExt, StreamExt};
use log::info;
use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};



#[tokio::main]
async fn main() -> Result<()>{
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    dotenv::dotenv().ok();
    let chat_port: u16 = env::var("CHAT_PORT").unwrap_or("8081".to_string()).parse().expect("ENV CHAT_PORT ERROR");
    // let url = env::var("DATABASE_URL").expect("ENV DATABASE_URL ERROR");
    // let web_port: u16 = env::var("WEB_PORT").unwrap_or("8080".to_string()).parse().expect("ENV WEB_PORT ERROR");
    let stream = TcpStream::connect(format!("127.0.0.1:{}", chat_port)).await?;
    info!("connect to chat server success");
    let (read, write) = stream.into_split();
    let mut framed = FramedRead::new(read, LengthDelimitedCodec::new());
    let (_sender, receiver) = tokio::sync::mpsc::channel::<String>(10);
    let mut frame_writer = FramedWrite::new(write, LengthDelimitedCodec::new());

    frame_writer.send(bytes::Bytes::from("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOjMsImV4cCI6MTczODQ4MTU1Mn0.PhQrbo2QCMmNQPZI0ensQmezw7n8RUxDWbhT2aM9w7I")).await?;

    tokio::spawn(async move {
        let mut receiver = receiver;
        while let Some(msg) = receiver.recv().await {
            match frame_writer.send(bytes::Bytes::from(msg)).await {
                Ok(_) => {}
                Err(e) => {
                    info!("send msg error:{}", e);
                }
            }
        }
    });
    // _sender.send("123".to_string()).await?;
    loop {
        match framed.next().await {
            Some(data) => {
                let logic_msg = String::from_utf8(data?.to_vec())?;
                info!("recv msg:{}", logic_msg);
            },
            None => {
                return Err(anyhow::anyhow!("read line from stream error"));
            },
        }
    }
}
