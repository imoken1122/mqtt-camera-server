use camera_driver::interface;
use camera_driver::interface::CameraInterface;
use camera_driver::mock::MockCamera;
use camera_driver::svb_camera;
use camera_driver::svb_camera::SVBCameraWrapper;
use env_logger;
use serde_json::error;
use std::time::Instant;

use log::{debug, error, info, warn};
use rumqttc::{self, AsyncClient, Event, MqttOptions, QoS};
use serde::{ Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::{task, };

const ResponceTopic: &str = "camera/responce";

fn gen_responce(
    camera_idx: &i32,
    cmd_idx: &i32,
    data: String,
) -> Result<String, serde_json::Error> {
    let mut res = HashMap::new();
    res.insert("camera_idx", camera_idx.to_string());
    res.insert("cmd_idx", cmd_idx.to_string());
    res.insert("data", data);
    let res_json = serde_json::to_string(&res);
    res_json
}

#[derive(Debug, Clone)]
pub enum Vendor {
    MOCK(Arc<Mutex<MockCamera>>),
    SVBONY(Arc<Mutex<SVBCameraWrapper>>),
}
#[derive(Debug, PartialEq, Eq)]
pub enum CameraCmd {
    GetInfo = 0,
    GetStatus,
    GetRoi,
    GetCtrlVal,
    SetRoi,
    SetCtrlVal,
    StartCapture,
    StopCapture,
    NotImplemented,
}
impl CameraCmd {
    fn from_i32(cmd_idx: &i32) -> CameraCmd {
        let cmd = match cmd_idx {
            0 => CameraCmd::GetInfo,
            1 => CameraCmd::GetStatus,
            2 => CameraCmd::GetRoi,
            3 => CameraCmd::GetCtrlVal,
            4 => CameraCmd::SetRoi,
            5 => CameraCmd::SetCtrlVal,
            6 => CameraCmd::StartCapture,
            7 => CameraCmd::StopCapture,
            _ => {
                error!("Unknown Payload value");
                CameraCmd::NotImplemented
            }
        };
        cmd
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    camera_idx: i32,
    cmd_idx: i32,
    data: HashMap<String, String>,
    // 他のフィールドも必要に応じて追加
}

#[derive(Debug, Clone)]
pub struct MQTTServer {
    client: AsyncClient,
}
impl MQTTServer {
    fn new(client: AsyncClient) -> Self {
        Self {
            client,
        }
    }

    async fn subscribe(&self, topics: &str) {
        self.client
            .subscribe(topics, QoS::ExactlyOnce)
            .await
            .unwrap_or_else(|e| {
                error!("Error subscribing to topics: {:?}", e);
            });
    }
    async fn publish(&self, topic: &str, payload: &str) {
        self.client
            .publish(topic, QoS::AtMostOnce, false, payload)
            .await
            .unwrap_or_else(|e| {
                error!("Error publishing message: {:?}", e);
            });
    }
    async fn get_frame<T: CameraInterface>(
        &self,
        camera: Arc<Mutex<T>>,
        buf: Arc<Mutex<Vec<String>>>,
    ) {
        while camera.lock().await.is_capture() {
            let encoded = camera.lock().await.get_frame();
            buf.lock().await.push(encoded);
        }
    }

pub async fn cmd_process<T: CameraInterface>(
    &mut self,
    camera: Arc<Mutex<T>>,
    dict: Payload,
) {
    let camera_idx = dict.camera_idx;
    let cmd_idx = dict.cmd_idx;
    let data = dict.data;

    let res_data = match CameraCmd::from_i32(&cmd_idx) {
        CameraCmd::GetInfo => {
            let info = camera.lock().await.get_info();
            let info_json = serde_json::to_string(&info);
            info!("[ MQTTServer ] : GetInfo command is executed by camera_idx = {:?}", camera_idx);
            info_json.unwrap()
        }
        CameraCmd::GetStatus => {
            //let status = camera.get_status();
            let status = r#"{"statuts"}"#;
            let status_json = serde_json::to_string(&status);
            status_json.unwrap()
        }
        CameraCmd::GetCtrlVal => {
            let ctrl_type_idx: i32 = data.get("ctrl_type").unwrap().parse().unwrap();
            let ctrl_type = interface::ControlType::from_i32(&ctrl_type_idx);
            let val = camera.lock().await.get_control_value(ctrl_type);
            let val_json = serde_json::to_string(&val);
            info!("[ MQTTServer ] GetCtrlVal command is executed by camera_idx = {:?}", camera_idx);
            val_json.unwrap()
        }
        CameraCmd::GetRoi => {
            let roi = camera.lock().await.get_roi();
            let roi_json = serde_json::to_string(&roi).unwrap();
            info!("[ MQTTServer ] : GetRoi command is executed by camera_idx = {:?}", camera_idx);
            roi_json
        }
        CameraCmd::SetCtrlVal => {
            let ctrl_type_idx: i32 = data.get("ctrl_type").unwrap().parse().unwrap();
            let ctrl_type = interface::ControlType::from_i32(&ctrl_type_idx);
            let value: i64 = data.get("value").unwrap().parse().unwrap();
            camera.lock().await.set_control_value(ctrl_type, value, false);
            info!("[ MQTTServer ] : SetCtrlVal command is executed by camera_idx = {:?}", camera_idx);
            r#"{}"#.to_string()
        }
        CameraCmd::SetRoi => {
            let startx: u32 = data.get("startx").unwrap().parse().unwrap();
            let starty: u32 = data.get("starty").unwrap().parse().unwrap();
            let width: u32 = data.get("width").unwrap().parse().unwrap();
            let height: u32 = data.get("height").unwrap().parse().unwrap();
            let bin: u8 = data.get("bin").unwrap().parse().unwrap();
            let img_type: i32 = data.get("img_type").unwrap().parse().unwrap();
            let img_type = interface::ImgType::from_i32(&img_type);
            camera
                .lock()
                .await
                .set_roi(startx, starty, width, height, bin, img_type);
            info!(
                "[ MQTTServer ] : SetRoi command is executed by camera_idx = {:?}",
                camera_idx);
            r#"{}"#.to_string()
        }
        CameraCmd::StartCapture => {
            camera.lock().await.start_capture();
            camera.lock().await.set_is_capture(true);
            info!(
                "[ MQTTServer ] : StartCapture command is executed by camera_idx = {:?}",
                camera_idx);
            while camera.lock().await.is_capture() {
                let buf = camera.lock().await.get_frame() ;
                let start = Instant::now();
                let mut res = HashMap::new();
                res.insert("frame", buf);
                let buf_json = serde_json::to_string(&res).unwrap();

                let res: String = gen_responce(&camera_idx, &cmd_idx, buf_json).unwrap();
                self.publish(ResponceTopic, &res).await;

                let end = Instant::now();
                let elapsed = end.duration_since(start);
                //debug!("Get frame time = {:?}", elapsed);
            }
            r#"{"frame"}"#.to_string()
        }
        CameraCmd::StopCapture => {
            camera.lock().await.set_is_capture(false);
            camera.lock().await.stop_capture();
            info!(
                "[ MQTTServer ] : StopCapture command is executed by camera_idx = {:?}",
                camera_idx);
            r#"{}"#.to_string()
        }

        CameraCmd::NotImplemented => {

            error!("[ MQTTServer ] : NotImplemented command is executed by camera_idx = {:?}", camera_idx);
            r#"{}"#.to_string()
        }
    };

    let res: String = gen_responce(&camera_idx, &cmd_idx, res_data).unwrap();
    self.publish(ResponceTopic, &res).await;
}
}
#[tokio::main(worker_threads = 10)]
async fn main() {
    //-> Result<(), Box<dyn Error>> {
    env_logger::init();
    let mut devices = Vec::new();
    devices.push(Vendor::MOCK(Arc::new(Mutex::new(MockCamera::new(0)))));
    devices.push(Vendor::MOCK(Arc::new(Mutex::new(MockCamera::new(1)))));
    let num_svb = svb_camera::num_svb();
    if num_svb > 0 {
        for i in 0..num_svb {
            devices.push(Vendor::SVBONY(Arc::new(Mutex::new(SVBCameraWrapper::new(
                i as usize,
            )))));
        }
    }

    let mut mqttoptions = MqttOptions::new("mqtt-server", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(20));
    mqttoptions.set_max_packet_size(100000000000, 1000000000000);
    let (client, mut eventloop) = AsyncClient::new(mqttoptions.clone(), 100000000000);
    let mut cli = MQTTServer::new(client);
    let cli_1 = cli.clone();
    task::spawn(async move {
        cli_1.subscribe("camera/instr").await;
    });

    while let Ok(event) = eventloop.poll().await {
        match event {
            Event::Incoming(pkt) => match pkt {
                rumqttc::Packet::Publish(pkt) => {
                    let topic = pkt.topic.as_str();
                    let payload = std::str::from_utf8(&pkt.payload).unwrap().to_owned();
                    let dict: Payload = serde_json::from_str(&payload).unwrap();
                    let camera_idx = dict.camera_idx;
                    let cmd_idx = dict.cmd_idx;

                    println!("topic = {:?}", topic);
                    println!("camera idx = {:?}", camera_idx);
                    println!("cmd = {:?}", dict.cmd_idx);
                    println!("data = {:?}", dict.data);

                    let camera = devices[camera_idx as usize].clone();
                    let mut cli_cln = cli.clone();
                    tokio::spawn(async move {
                        match camera {
                            Vendor::SVBONY(ref svb) => {
                                let svb = svb.clone();
                                cli_cln.cmd_process( svb, dict).await;
                            }
                            Vendor::MOCK(ref mock) => {
                                let mock = mock.clone();
                                cli_cln.cmd_process( mock, dict).await;
                            }
                            _ => panic!("Unknown camera type"),
                        };
                    });
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
