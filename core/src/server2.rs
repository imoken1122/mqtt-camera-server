use camera_driver::interface::CameraInterface;
use tokio::{task, time};

use camera_driver::mock::MockCamera;
use rumqttc::{self, AsyncClient, Event, MqttOptions, QoS};
use serde::{Deserialize, Serialize, de};
use std::collections::HashMap;
use std::error::Error;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::vec;

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
struct MQTTServer<T: CameraInterface> {
    client: AsyncClient,
    devices: Vec<Arc<Mutex<T>>>,
}
impl<T> MQTTServer<T>
where
    T: CameraInterface,
{
    fn new(client: AsyncClient, devices: Vec<Arc<Mutex<T>>>) -> Self {
        Self {
            client,
           devices
        }
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
    //fn process_camera_cmd(&self,cmd : )
    async fn run(&self, eventloop: &mut rumqttc::EventLoop) -> Result<(), Box<dyn Error>>{
        while let Ok(event) = eventloop.poll().await {
            match event {
                Event::Incoming(pkt) => match pkt {
                    rumqttc::Packet::Publish(v) => {
                        let topic = v.topic;
                        let payload = std::str::from_utf8(&v.payload).unwrap().to_owned();
                        let dict: Command = serde_json::from_str(&payload).unwrap();
                        let camera_idx = dict.camera_idx;

                        println!("topic = {:?}", topic);
                        println!("idx = {:?}", camera_idx);
                        println!("camera = {:?}", self.devices[camera_idx as usize].lock().unwrap().get_info());
                        println!("data = {:?}", dict.data);
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
        Ok(())
    }
}

#[tokio::main(worker_threads = 1)]
async fn main() {
    //-> Result<(), Box<dyn Error>> {
    // color_backtrace::install();
    let devices = vec![Arc::new(Mutex::new(MockCamera::new(0))), Arc::new(Mutex::new(MockCamera::new(1)))];

    let mut mqttoptions = MqttOptions::new("test-1", "localhost", 1883);
        mqttoptions.set_keep_alive(Duration::from_secs(5));

        let (client, mut eventloop) = AsyncClient::new(mqttoptions.clone(), 10);
        
        let cli = MQTTServer::new(client, devices);
        let cli_1 = cli.clone();
        tokio::spawn(async move {
            cli_1
                .subscribe(format!("camera/instr" ).as_str())
                .await;

        });

       let t = tokio::spawn( async move  {cli.run(&mut eventloop).await;});
       t.await.unwrap();

}
