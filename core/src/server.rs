///
///
/// MQTT Camera Server
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
///
use camera_driver::interface;
use camera_driver::interface::CameraInterface;
use camera_driver::mock::MockCamera;
use camera_driver::svb_camera;
use camera_driver::svb_camera::SVBCameraWrapper;
use env_logger;
use serde_json::error;
use std::hash::Hash;
use std::time::Instant;

use log::{debug, error, info, warn};
use rumqttc::{self, AsyncClient, Event, MqttOptions, QoS};
use serde::{de, Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task;

const ResponceTopic: &str = "camera/responce";
const InitTopic: &str = "camera/init";

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
    Init,
    NotImplemented = -1,
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
            8 => CameraCmd::Init,
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
    transaction_id: String,
    camera_idx: i32,
    cmd_idx: i32,
    data: HashMap<String, String>,
}

fn get_devices() -> Vec<Vendor> {
    let mut devices = Vec::new();
    let num_mock = MockCamera::num_devices();
    if num_mock > 0 {
        for i in 0..num_mock {
            devices.push(Vendor::MOCK(Arc::new(Mutex::new(MockCamera::new(i)))));
        }
    }

    let num_svb = SVBCameraWrapper::num_devices();
    if num_svb > 0 {
        for i in 0..num_svb {
            devices.push(Vendor::SVBONY(Arc::new(Mutex::new(SVBCameraWrapper::new(
                i as usize,
            )))));
        }
    }
    devices
}
async fn close_devices(devices: &Vec<Vendor>) {
    for device in devices {
        match device {
            Vendor::MOCK(mock) => {
                mock.lock().await.close();
            }
            Vendor::SVBONY(svb) => {
                svb.lock().await.close();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MQTTCameraServer {
    client: AsyncClient,
}
impl MQTTCameraServer {
    fn new(client: AsyncClient) -> Self {
        Self { client }
    }
    fn gen_responce(
        &self,
        t_id: &String,
        camera_idx: &i32,
        cmd_idx: &i32,
        data: String,
    ) -> Result<String, serde_json::Error> {
        let mut res = HashMap::new();
        res.insert("transaction_id", t_id.to_string());
        res.insert("camera_idx", camera_idx.to_string());
        res.insert("cmd_idx", cmd_idx.to_string());
        res.insert("data", data);
        let res_json = serde_json::to_string(&res);
        res_json
    }
    fn to_json(&self, responce: &HashMap<String, String>) -> Result<String, serde_json::Error> {
        let res_json = serde_json::to_string(&responce);
        res_json
    }
    // Subscribes to the topic
    async fn subscribe(&self, topics: &str) {
        self.client
            .subscribe(topics, QoS::ExactlyOnce)
            .await
            .unwrap_or_else(|e| {
                error!("Error subscribing to topics: {:?}", e);
            });
    }
    // Publishes a message to the topic
    async fn publish(&self, topic: &str, payload: &str) {
        self.client
            .publish(topic, QoS::AtLeastOnce, false, payload)
            .await
            .unwrap_or_else(|e| {
                error!("Error publishing message: {:?}", e);
            });
    }

    // The process is executed according to the command index extracted from the payload.
    pub async fn cmd_process<T: CameraInterface>(&mut self, camera: Arc<Mutex<T>>, dict: Payload) {
        let transaction_id = dict.transaction_id;
        let camera_idx = dict.camera_idx;
        let cmd_idx = dict.cmd_idx;
        let mut data = dict.data;

        let res_data = match CameraCmd::from_i32(&cmd_idx) {
            CameraCmd::GetInfo => {
                //
                // incoming and outcoming data field  :
                // {    name,
                //      idx,
                //      max_width,
                //      max_height,
                //      supported_img_type,
                //      supported_bins,
                //      is_coolable
                // }

                let info = camera.lock().await.get_info();
                let info_json = serde_json::to_string(&info);
                info!(
                    "[ MQTTServer ] : GetInfo command is executed by camera_idx = {:?}",
                    camera_idx
                );
                info_json.unwrap()
            }
            CameraCmd::GetStatus => {
                //let status = camera.get_status();
                let status = r#"{"statuts"}"#;
                let status_json = serde_json::to_string(&status);
                status_json.unwrap()
            }
            CameraCmd::GetCtrlVal => {
                // incoming and outcoming data field  :
                // {    ctrl_type
                //    value
                // }

                let ctrl_type_idx: i32 = data.get("ctrl_type").unwrap().parse().unwrap();
                let ctrl_type = interface::ControlType::from_i32(&ctrl_type_idx);
                let val = camera.lock().await.get_control_value(ctrl_type);

                data.insert("value".to_string(), val.to_string());
                let val_json = serde_json::to_string(&data);
                info!(
                    "[ MQTTServer ] GetCtrlVal command is executed by camera_idx = {:?}",
                    camera_idx
                );
                val_json.unwrap()
            }
            CameraCmd::GetRoi => {
                //
                // incoming and outcoming data field  :
                // {    startx,
                //      starty,
                //      width,
                //      height,
                //      bin,
                //      img_type
                // }
                let roi = camera.lock().await.get_roi();
                let roi_json = serde_json::to_string(&roi).unwrap();
                info!(
                    "[ MQTTServer ] : GetRoi command is executed by camera_idx = {:?}",
                    camera_idx
                );
                roi_json
            }
            CameraCmd::SetCtrlVal => {
                // Return ctrl value  after set control value
                //
                // incoming and outcoming data field  :
                // {
                //      ctrl_type : int,
                //      value : int
                // }

                let ctrl_type_idx: i32 = data.get("ctrl_type").unwrap().parse().unwrap();
                let ctrl_type = interface::ControlType::from_i32(&ctrl_type_idx);
                let value: i64 = data.get("value").unwrap().parse().unwrap();
                camera
                    .lock()
                    .await
                    .set_control_value(ctrl_type, value, false);
                info!(
                    "[ MQTTServer ] : SetCtrlVal command is executed by camera_idx = {:?}",
                    camera_idx
                );

                let val = camera.lock().await.get_control_value(ctrl_type);
                let mut res = HashMap::new();
                res.insert("value".to_string(), val.to_string());
                res.insert("ctrl_type".to_string(), ctrl_type_idx.to_string());

                let ctrl_json = serde_json::to_string(&res).unwrap();
                ctrl_json
            }
            CameraCmd::SetRoi => {
                // Return ROI after set ROI
                // responce data field  :
                // {    startx : int,
                //      starty : int,
                //      width : int ,
                //      height : int,
                //      bin : int,
                //      img_type : int
                // }

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
                    camera_idx
                );
                let roi = camera.lock().await.get_roi();
                let roi_json = serde_json::to_string(&roi).unwrap();
                roi_json
            }
            CameraCmd::StartCapture => {
                //
                // responce data field  :
                // {
                //       frame : base64 encoded raw data
                // }
                //
                // The camera starts capturing and returns the frame data.
                // The frame data is encoded in base64
                // keep to catpure and publish frame data until StopCapture command is executed.
                //
                camera.lock().await.start_capture();
                camera.lock().await.set_is_capture(true);
                info!(
                    "[ MQTTServer ] : StartCapture command is executed by camera_idx = {:?}",
                    camera_idx
                );
                while camera.lock().await.is_capture() {
                    let buf = camera.lock().await.get_frame();
                    let start = Instant::now();
                    let mut res = HashMap::new();
                    res.insert("frame", buf);
                    let buf_json = serde_json::to_string(&res).unwrap();

                    let res: String = self
                        .gen_responce(&transaction_id, &camera_idx, &cmd_idx, buf_json)
                        .unwrap();
                    self.publish(ResponceTopic, &res).await;

                    let end = Instant::now();
                    let elapsed = end.duration_since(start);
                    //debug!("Get frame time = {:?}", elapsed);
                }
                r#"{}"#.to_string()
            }
            CameraCmd::StopCapture => {
                //
                //  camera stop capturing and set is_capture = false

                camera.lock().await.set_is_capture(false);
                camera.lock().await.stop_capture();
                info!(
                    "[ MQTTServer ] : StopCapture command is executed by camera_idx = {:?}",
                    camera_idx
                );
                r#"{}"#.to_string()
            }

            CameraCmd::NotImplemented => {
                error!(
                    "[ MQTTServer ] : NotImplemented command is executed by camera_idx = {:?}",
                    camera_idx
                );
                r#"{}"#.to_string()
            }
            _ => {
                error!(
                    "[ MQTTServer ] : Unknown command or not using command is executed by camera_idx = {:?}",
                    camera_idx
                );
                r#"{}"#.to_string()
            }
        };

        let res: String = self
            .gen_responce(&transaction_id, &camera_idx, &cmd_idx, res_data)
            .unwrap();
        self.publish(ResponceTopic, &res).await;
    }
}
#[tokio::main(worker_threads = 10)]
async fn main() {
    //-> Result<(), Box<dyn Error>> {
    env_logger::init();

    // The mqtt server is established.
    let mut mqttoptions = MqttOptions::new("mqtt-server", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(20));
    mqttoptions.set_max_packet_size(100000000000, 1000000000000);
    let (client, mut eventloop) = AsyncClient::new(mqttoptions.clone(), 100000000000);
    let cli = MQTTCameraServer::new(client);
    let cli_1 = cli.clone();

    // Get all connected cameras.
    //let mut devices = get_devices();
    let mut devices: Vec<Vendor> = Vec::new();

    task::spawn(async move {
        cli_1.subscribe("camera/instr").await;
        cli_1.subscribe("camera/init").await;
    })
    .await
    .unwrap();

    // This mqtt server receives messages from the mqtt client, and the camera executes the process according to the command index extracted in the payload.
    while let Ok(event) = eventloop.poll().await {
        match event {
            Event::Incoming(pkt) => match pkt {
                rumqttc::Packet::Publish(pkt) => {
                    let topic = pkt.topic.as_str();
                    let payload = std::str::from_utf8(&pkt.payload).unwrap().to_owned();
                    let dict: Payload = serde_json::from_str(&payload).unwrap();
                    let camera_idx = dict.camera_idx;
                    let cmd_idx = dict.cmd_idx;
                    let t_id = &dict.transaction_id;

                    info!("[ MQTTServer] ====== Received Payload =======");
                    info!("[ MQTTServer] Topic:            {}", topic);
                    info!("[ MQTTServer] Camera index:     {}", camera_idx);
                    info!("[ MQTTServer] Command received: {}", cmd_idx);
                    info!("[ MQTTServer] Data received:    {:?}", dict.data);

                    match topic {
                        // init topic is get number of connected camera
                        "camera/init" => {
                            debug!("[ MQTTServer ] : Init publish to {} ", InitTopic);
                            close_devices(&devices).await;
                            devices = get_devices();

                            let mut data = HashMap::new();
                            data.insert("num_device".to_string(), devices.len().to_string());
                            let data = cli.to_json(&data).unwrap();
                            let res_json = cli.gen_responce(t_id, &-1, &8, data).unwrap();
                            cli.publish(ResponceTopic, &res_json).await;
                        }
                        // instr topic is get camera command and execute command
                        "camera/instr" => {
                            let camera = devices[camera_idx as usize].clone();
                            let mut cli_cln = cli.clone();

                            // The process is executed asynchronously by the tokio library.
                            tokio::spawn(async move {
                                match camera {
                                    Vendor::SVBONY(ref svb) => {
                                        let svb = svb.clone();
                                        cli_cln.cmd_process(svb, dict).await;
                                    }
                                    Vendor::MOCK(ref mock) => {
                                        let mock = mock.clone();
                                        cli_cln.cmd_process(mock, dict).await;
                                    }
                                    _ => error!("[ MQTTServer] Unknown camera vendor"),
                                };
                            });
                        }
                        _ => {
                            error!("[ MQTTServer] Unknown topic");
                        }
                    }
                }
                _ => {
                    debug!("[ MQTTServer] Other packet : {:?}", pkt);
                }
            },

            Event::Outgoing(v) => {
                info!("[ MQTTServer] Outgoing = {:?}", v);
            }
        }
    }
}
