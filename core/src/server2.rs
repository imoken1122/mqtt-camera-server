use camera_driver::interface::CameraInterface;
use camera_driver::interface;
use tokio::{task, time};

use camera_driver::mock::MockCamera;
use camera_driver::svb_camera::SVBCameraWrapper;
use camera_driver::svb_camera; 
use rumqttc::{self, AsyncClient, Event, MqttOptions, QoS};
use serde::{de, Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;
use std::{clone, vec};
use tokio::sync::Mutex;
use log::{debug, error, info, warn};

const ResponceTopic : &str = "camera/responce"; 

fn gen_responce(camera_idx : &i32, cmd_idx : &i32, data :String) -> Result<String,serde_json::Error>{
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
enum CameraCmd {
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
struct Payload {
    camera_idx: i32,
    cmd_idx: i32,
    data: HashMap<String, String>,
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
    async fn cmd_process<T:CameraInterface>(&self, camera : Arc<Mutex<T>> , dict : Payload){
        let camera_idx = dict.camera_idx;
        let cmd_idx = dict.cmd_idx;
        let data = dict.data;

        let res_data   = match CameraCmd::from_i32(&cmd_idx){

            CameraCmd::GetInfo => {
                let info = camera.lock().await.get_info();
                let info_json = serde_json::to_string(&info);
                info_json.unwrap()

            }
            CameraCmd::GetStatus => {
                //let status = camera.get_status();
                let status = r#"{"statuts"}"#;
                let status_json = serde_json::to_string(&status);
                status_json.unwrap()
            }
            CameraCmd::GetCtrlVal => {
                let ctrl_type_idx : i32 = data.get("ctrl_type").unwrap().parse().unwrap();
                let ctrl_type = interface::ControlType::from_i32(&ctrl_type_idx);
                let val = camera.lock().await. get_control_value(ctrl_type);
                let val_json = serde_json::to_string(&val);
                val_json.unwrap()
            }
            CameraCmd::GetRoi => {
                let roi = camera.lock().await.get_roi();
                let roi_json = serde_json::to_string(&roi).unwrap();
                roi_json
            }
            CameraCmd::SetCtrlVal => {
                let ctrl_type_idx : i32 = data.get("ctrl_type").unwrap().parse().unwrap();
                let ctrl_type = interface::ControlType::from_i32(&ctrl_type_idx);
                let value : i64= data.get("value").unwrap().parse().unwrap();
                camera.lock().await.set_control_value(ctrl_type,value,0);
                r#"{}"#.to_string()
            }
            CameraCmd::SetRoi =>{
                let startx : u32 = data.get("startx").unwrap().parse().unwrap();
                let starty : u32 = data.get("starty").unwrap().parse().unwrap();
                let width : u32 = data.get("width").unwrap().parse().unwrap();
                let height : u32 = data.get("height").unwrap().parse().unwrap();
                let bin : u8 = data.get("bin").unwrap().parse().unwrap();
                let img_type : i32 = data.get("img_type").unwrap().parse().unwrap();
                let img_type = interface::ImgType::from_i32(&img_type);
                camera.lock().await.set_roi(startx,starty,width,height,bin,img_type);
                r#"{}"#.to_string()
            }
            CameraCmd::StartCapture => {
                camera.lock().await.start_capture();
                r#"{}"#.to_string()
            }
            CameraCmd::StopCapture => {
                camera.lock().await.stop_capture();
                r#"{}"#.to_string()
            }
                
        CameraCmd::NotImplemented => {
            error!("Not implemented");
            r#"{}"#.to_string()
        }
    };                

        let res : String = gen_responce(&camera_idx , &cmd_idx, res_data).unwrap();
        self.publish(ResponceTopic,&res).await;
    }
}

#[tokio::main()]
async fn main() {
    //-> Result<(), Box<dyn Error>> {
    let mut devices = Vec::new();
    let num_svb = svb_camera::num_svb();
    devices.push(Vendor::MOCK(Arc::new(Mutex::new(MockCamera::new(0)))));
    devices.push(Vendor::SVBONY(Arc::new(Mutex::new(SVBCameraWrapper::new(0)))));

    let mut mqttoptions = MqttOptions::new("test-1", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    let (client, mut eventloop) = AsyncClient::new(mqttoptions.clone(), 20);
    let cli = MQTTServer::new(client);
    let cli_1 = cli.clone();
    task::spawn(async move {
        cli_1.subscribe(format!("camera/instr").as_str()).await;
        time::sleep(Duration::from_secs(1)).await;
    });


    while let Ok(event) = eventloop.poll().await {
        match event {
            Event::Incoming(pkt) => match pkt {
                rumqttc::Packet::Publish(pkt) => {
                    let topic = pkt.topic.as_str();
                    let payload = std::str::from_utf8(&pkt.payload).unwrap().to_owned();
                    let dict: Payload = serde_json::from_str(&payload).unwrap();
                    let camera_idx = dict.camera_idx;


                    println!("topic = {:?}", topic);
                    println!("camera idx = {:?}", camera_idx);
                    println!("data = {:?}", dict.data);

                    let camera = devices[camera_idx as usize].clone();
                    let cli_cln= cli.clone();
                    match camera {
                        Vendor::SVBONY(ref svb) => {
                            let svb = svb.clone();
                            task::spawn(async move {
                                cli_cln.cmd_process(svb, dict).await;
                            });
                        }
                        Vendor::MOCK(ref mock) => {
                            let mock = mock.clone();
                            task::spawn(async move {
                                cli_cln.cmd_process(mock, dict).await;
                            });
                        }
                        _ => panic!("Unknown camera type"),
                    };
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
