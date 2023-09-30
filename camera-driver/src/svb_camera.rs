use crate::interface::{CameraInfo, CameraInterface, ControlType, ImgType, ROIFormat};

use log::error;
use serde::{Deserialize, Serialize};
use svbony_camera_rs::{camera as svb, libsvb};

use base64::encode;
use std::sync;
pub fn num_svb() -> i32 {
    svb::get_num_of_camera()
}

#[derive(Debug, Clone)]
pub struct SVBCameraWrapper {
    camera: svb::Camera,
    roi: ROIFormat,
    info: CameraInfo,
    is_capture: bool,
}

impl CameraInterface for SVBCameraWrapper {
    fn new(idx: usize) -> Self {
        let mut camera = svb::Camera::new(idx as i32);
        camera.init();

        let roi = camera.roi;
        let img_type = camera.get_img_type().unwrap();

        let roi = ROIFormat {
            startx: roi.startx as u32,
            starty: roi.starty as u32,
            width: roi.width as u32,
            height: roi.height as u32,
            bin: roi.bin as u8,
            img_type: img_type as u8,
        };

        let info = camera.info;
        let props = camera.prop;

        let name: Vec<u8> = info.FriendlyName.iter().map(|&x| x as u8).collect();
        let info = CameraInfo {
            name: String::from_utf8_lossy(&name).replace("\\u0000", ""),
            idx: idx as u32,
            max_width: props.MaxWidth as u32,
            max_height: props.MaxHeight as u32,
            supported_img_type: props
                .SupportedVideoFormat
                .iter()
                .take_while(|&x| *x != -1)
                .map(|x| ImgType::from_i32(x))
                .collect(),
            supported_bins: props.SupportedBins.iter().map(|x| *x as u8).collect(),
            is_coolable: false,
        };

        camera.adjust_white_blance();
        camera.set_ctl_value(libsvb::SVB_CONTROL_TYPE_SVB_FLIP, 3, 0);

        SVBCameraWrapper {
            camera,
            roi,
            info,
            is_capture: false,
        }
    }

    fn start_capture(&mut self) {
        self.camera.start_video_capture();
        self.is_capture = true
    }
    fn stop_capture(&mut self) {
        self.camera.stop_video_capture();
        self.is_capture = false
    }
    fn get_frame(&self) -> String {
        let buf = self.camera.get_video_frame().unwrap();
        encode(buf)
    }
    fn close(&self) {
        self.camera.close();
    }
    fn get_info(&self) -> CameraInfo {
        self.info.clone()
    }
    fn get_roi(&self) -> ROIFormat {
        self.roi.clone()
    }

    fn set_control_value(&self, ctrl_type: ControlType, value: i64, is_auto: bool) {
        let svb_ctrl_type = ControlType::to_svb(ctrl_type);
        self.camera
            .set_ctl_value(svb_ctrl_type, value, is_auto as u32);
    }
    fn get_control_value(&self, ctrl_type: ControlType) -> i64 {
        let svb_ctrl_type = ControlType::to_svb(ctrl_type);
        match self.camera.get_ctl_value(svb_ctrl_type) {
            Ok(state) => state.value,
            Err(e) => {
                error!("get_control_value error: {:?}", e);
                -1
            }
        }
    }
    fn get_img_type(&self) -> ImgType {
        let svb_img_t = self.camera.get_img_type().unwrap();
        ImgType::from_i32(&svb_img_t)
    }
    fn set_img_type(&mut self, img_type: ImgType) {}
    fn is_capture(&self) -> bool {
        self.is_capture
    }
    fn set_is_capture(&mut self, is_capture: bool) {
        self.is_capture = is_capture
    }
    fn set_roi(
        &mut self,
        startx: u32,
        starty: u32,
        width: u32,
        height: u32,
        bin: u8,
        img_type: ImgType,
    ) {
            let svb_img_type = ImgType::to_svb(img_type);
              self.camera.set_roi_format(startx as i32, starty as i32, width as i32, height as i32, bin as i32);
              self.camera.set_img_type(svb_img_type);
    }
}
