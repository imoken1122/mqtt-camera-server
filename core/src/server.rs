use async_std;
use camera_driver::interface::{self, CameraInterface};
use camera_driver::mock::MockCamera;
use futures::executor::block_on;
use futures::StreamExt;
use log::{debug, error, info, warn};
use mqtt::string_collection;
use paho_mqtt as mqtt;
use std::collections::HashMap;
use std::hash::Hash;
use std::thread;
use std::time::Duration;
use std::{env, process};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json;

fn gen_responce(camera_idxx : &i32, cmd_idx : &i32, data :String) -> Result<String,serde_json::Error>{
    let mut res = HashMap::new();
    res.insert("camera_idxx", camera_idxx.to_string());
    res.insert("cmd_idx", cmd_idx.to_string());
    res.insert("data", data);
    let res_json = serde_json::to_string(&res);
    res_json
}

#[derive(Debug, Deserialize, Serialize)]
struct CommandData {
    camera_idxx: i32,
    cmd_idx: i32,
    data: String,
}

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
    NotImplemented
}
impl CameraCmd {
    fn from_i32(cmd_idx: i32) -> CameraCmd {
        let cmd = match cmd_idx {
            1 => CameraCmd::GetInfo,
            2 => CameraCmd::GetProps,
            3 => CameraCmd::GetStatus,
            4 => CameraCmd::GetRoi,
          //  5 => CameraCmd::SetRoi,
          //  6 => CameraCmd::GetCtrlVal,
          //  7 => CameraCmd::SetCtrlVal,
          //  8 => CameraCmd::StartCapture,
          //  9 => CameraCmd::StopCapture,
            _ => {
                error!("Unknown command value");
                CameraCmd::NotImplemented
            }
        };
        cmd
    }
}

struct MQTTCameraServer{
    client: mqtt::AsyncClient,
}

impl MQTTCameraServer {
    pub fn new() -> Self {
        let host = env::args()
            .nth(1)
            .unwrap_or_else(|| "tcp://localhost:1883".to_string());
        info!("Connecting to the MQTT server at '{}'", host);

        let create_opts = mqtt::CreateOptionsBuilder::new_v3()
            .server_uri(&host)
            .client_id("async-subscriber")
            .finalize();

        let client = mqtt::AsyncClient::new(create_opts).unwrap_or_else(|err| {
            error!("Error creating the client: {:?}", err);
            process::exit(1);
        });
        Self { client }
    }
    pub fn connect_callback(&self) {
        self.client
            .set_connected_callback(|_cli: &mqtt::AsyncClient| {
                info!("connected ");
            });
    }
    pub fn disconnect_callback(&self) {
        self.client
            .set_connection_lost_callback(|cli: &mqtt::AsyncClient| {
                info!("disconnected. Attempting reconnect.");
                thread::sleep(Duration::from_millis(2500));

                cli.reconnect();
            });
    }
    pub fn on_message_callback<T:CameraInterface +Send>(&self, camera : CameraWrapper<T>) {
        let camera = Arc::clone(&camera.inner);
        self.client.set_message_callback(move|cli, msg| {
            if let Some(msg) = msg {
                let payload_str = msg.payload_str().into_owned();
                let dict: CommandData= serde_json::from_str(&payload_str).unwrap();

                let camera_idx = dict.camera_idxx;
                let cmd_idx = dict.cmd_idx;
                let res_data   = match CameraCmd::from_i32(cmd_idx) {

                    CameraCmd::GetInfo => {
                        //let info = camera.get_info();
                        //let info_json = serde_json::to_string(&info);
                        let info = r#"{}"#;
                        let info_json = serde_json::to_string(&info);
                        info_json.unwrap()

                    }
                    CameraCmd::GetProps => {
                        //let props = camera.get_props();
                        let props = r#"{"props"}"#;
                        let props_json = serde_json::to_string(&props) ;
                        props_json.unwrap()
                    }
                    CameraCmd::GetStatus => {
                        //let status = camera.get_status();
                        let status = r#"{"statuts"}"#;
                        let status_json = serde_json::to_string(&status);
                        status_json.unwrap()
                    }
                //    CameraCmd::GetCtrlVal => {
                //        let dict: HashMap<String, u8> = serde_json::from_str(&dict.data).unwrap();
                //        let ctrl_type_idx = dict.get("ctrl_type").unwrap();
                //        let ctrl_type = interface::ControlType::from_u8(ctrl_type_idx);
                //        let val = camera. get_control_value(ctrl_type);
                //        let val_json = serde_json::to_string(&val);
                //        val_json.unwrap()
                //    }
                    CameraCmd::GetRoi => {
                        let roi = camera.lock().unwrap().get_roi();
                        let roi_json = serde_json::to_string(&roi).unwrap();
                        roi_json
                    }
                //    CameraCmd::SetCtrlVal => {
                //        let dict: HashMap<String, i64> = serde_json::from_str(&dict.data).unwrap();
                //        let ctrl_type_idx = *dict.get("ctrl_type").unwrap() as u8;
                //        let ctrl_type = interface::ControlType::from_u8(&ctrl_type_idx);
                //        camera.set_control_value(
                //                                ctrl_type,
                //                                *dict.get("value").unwrap(),
                //                                *dict.get("is_auto").unwrap() as i32,
                //                            );
                //        r#"{}"#.to_string()
                //        
                //        
                //    }
                //    CameraCmd::SetRoi => {
                //        let roi : interface::ROIFormat = serde_json::from_str(&dict.data).unwrap();
                //        let img_type = interface::ImgType::from_u8(roi.img_type );
                //        let roi = camera.set_roi(roi.startx, roi.starty, roi.width, roi.height, roi.bin, img_type);
                //        r#"{}"#.to_string()

                //    }
                //    CameraCmd::StartCapture => {
                //        camera.start_capture();
                //        r#"{}"#.to_string()
                //    }
                //    CameraCmd::StopCapture => {
                //        camera.stop_capture();
                //        r#"{}"#.to_string()
                //    }
                    CameraCmd::NotImplemented => {
                        error!("Not implemented");
                        r#"{}"#.to_string()
                    }
                };
            let res : String = gen_responce(&camera_idx, &cmd_idx, res_data).unwrap();
            let msg = mqtt::Message::new("camera/responce", res, mqtt::QOS_1);
            cli.publish(msg);
            }
        });
    }
}
struct CameraWrapper<T : CameraInterface> {
    inner : Arc<Mutex<T>>
}
fn main() {
    env_logger::init();
    let  camera = camera_driver::mock::MockCamera::new(0);
    let mut arc_camera = Arc::new(Mutex::new(camera));
    let wrapper = CameraWrapper{inner: arc_camera};
    let mut server = MQTTCameraServer::new();
    server.connect_callback();
    server.disconnect_callback();
    server.on_message_callback(wrapper);

}