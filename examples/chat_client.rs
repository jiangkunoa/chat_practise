use std::env;

use anyhow::Result;
use futures::{SinkExt, StreamExt};
use log::info;
use tokio::{io::{stdin, AsyncReadExt}, net::TcpStream};
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
            match frame_writer.send(bytes::Bytes::from(msg.clone())).await {
                Ok(_) => {
                    info!("send msg success:{}", msg);
                }
                Err(e) => {
                    info!("send msg error:{}", e);
                }
            }
        }
    });
    // _sender.send("123".to_string()).await?;
    tokio::spawn(async move {
        loop {
            match framed.next().await {
                Some(data) => {
                    let logic_msg = String::from_utf8(data.unwrap().to_vec()).unwrap();
                    info!("recv msg:{}", logic_msg);
                },
                None => {
                    println!("read line from stream error");
                },
            }
        }
    });
    loop {
        //读取控制台输入
        let mut input = String::new();
        info!("Please input cmd:");
        input.clear();
        stdin().read_to_string(&mut input).await.expect("read input error");
        //解析input为vec
        let input_vec: Vec<&str> = input.split(' ').collect();
        let cmd = input_vec.iter().next().ok_or_else(|| anyhow::anyhow!("cmd error"))?;
        match cmd.to_string().as_str() {
            "exit" => {
                info!("exit");
                break;
            },
            _ => {
                info!("unknown cmd:{}", cmd);
            }
        }
    }
    Ok(())
    
}
