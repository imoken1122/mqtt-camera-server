use camera_driver::interface::CameraInterface;
use tokio::{task, time};

use camera_driver::mock::MockCamera;
use rumqttc::{self, AsyncClient, Event, MqttOptions, QoS};
use serde::{de, Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use std::{clone, vec};
use tokio::sync::Mutex;
#[derive(Debug, PartialEq, Eq)]
enum CameraCmd {
    GetInfo,
    GetProps,
    GetStatus,
    GetRoi,
    //  SetRoi,
    //  GetCtrlVal,
    //  SetCtrlVal,
    //  StartCapture,
    //  StopCapture,
    NotImplemented,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Command {
    camera_idx: u8,
    cmd: String,
    data: HashMap<String, i64>,
    // 他のフィールドも必要に応じて追加
}

#[derive(Debug, Clone)]
struct MQTTServer {
    client: AsyncClient,
}
impl MQTTServer {
    fn new(client: AsyncClient) -> Self {
        Self { client }
    }

    async fn subscribe(&self, topics: &str) {
        self.client
            .subscribe(topics, QoS::AtMostOnce)
            .await
            .unwrap();
    }
    async fn publish(&self, topic: &str, payload: &str) {
        self.client
            .publish(topic, QoS::ExactlyOnce, false, payload)
            .await
            .unwrap();
    }
}

#[tokio::main()]
async fn main() {
    //-> Result<(), Box<dyn Error>> {
    let devices = vec![
        Arc::new(Mutex::new(MockCamera::new(0))),
        Arc::new(Mutex::new(MockCamera::new(1))),
    ];
    let mut mqttoptions = MqttOptions::new("test-1", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    let (client, mut eventloop) = AsyncClient::new(mqttoptions.clone(), 20);
    let cli = MQTTServer::new(client);
    let cli_1 = cli.clone();
    let cli_2 = cli.clone();
    task::spawn(async move {
        cli_1.subscribe(format!("camera/0/instr").as_str()).await;
        time::sleep(Duration::from_secs(1)).await;
    });

    task::spawn(async move {
        cli_2.subscribe(format!("camera/1/instr").as_str()).await;
        time::sleep(Duration::from_secs(1)).await;
    });

    while let Ok(event) = eventloop.poll().await {
        match event {
            Event::Incoming(pkt) => match pkt {
                rumqttc::Packet::Publish(v) => {
                    let topic = v.topic.as_str();
                    let payload = std::str::from_utf8(&v.payload).unwrap().to_owned();
                    let dict: Command = serde_json::from_str(&payload).unwrap();
                    let camera_idx = dict.camera_idx;

                    println!("topic = {:?}", topic);
                    println!("idx = {:?}", camera_idx);
                    println!("data = {:?}", dict.data);
                    match topic {
                        "camera/0/instr" => {
                            println!("camera 1");
                            let d = devices[0].clone();
                            let t = task::spawn(async move {
                                let v = d.lock().await.get_frame();
                            });
                        }

                        "camera/1/instr" => {
                            println!("camera 2");

                            let d = devices[1].clone();
                            let t = task::spawn(async move {
                                let v = d.lock().await.get_frame();
                            });
                        }
                        _ => {
                            println!("non topic");
                        }
                    }
                }
                _ => {
                    println!("not instr = {:?}", pkt);
                }
            },
            Event::Outgoing(v) => {
                println!("Outgoing = {:?}", v);
            }
        }
    }
}
