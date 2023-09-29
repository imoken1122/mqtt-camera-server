use svbony_camera_rs::camera as svb;
use crate::interface::{CameraInterface, CameraInfo, ControlType, ImgType, ROIFormat};

pub fn num_svbcamera() -> usize {
    svb::get_num_of_camera() as usize
}
impl CameraInterface for svb::Camera{
    fn new(idx: usize) -> Self {
       let mut camera = svb::Camera::new(idx as i32);
       camera.init();
       camera

    }
    fn set_roi(&mut self, startx: u32, starty: u32, width: u32, height: u32, bin: u8, img_type: ImgType) {
        self.set_roi(startx, starty, width, height, bin);
        self.set_img_type(img_type);
    }

    fn set_img_type(&mut self, img_type: ImgType) {
        // set_img_typeメソッドの実装
    }

    fn get_roi(&self) -> ROIFormat {
        // roiの適当な要素を生成
        ROIFormat {
            startx: 0,
            starty: 0,
            width: 0,
            height: 0,
            bin: 0,
            img_type: 0,
        }
    }

    fn get_img_type(&self) -> ImgType {
        // imgtypの適当な要素を生成
        ImgType::RAW8
    }

    fn start_capture(&mut self) {
        self.is_capture = true;
    }

    fn stop_capture(&mut self) {
        self.is_capture = false;
    }
}