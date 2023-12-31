use crate::interface::{CameraInfo, CameraInterface, ControlType, ImgType, ROIFormat};
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use rand::Rng; // ランダムな値を生成するために使用
use std::thread;
use std::time::Duration;
use tokio;

// CameraInfo、ROIFormat、ImgType、ControlType、ControlCapsなどのデータ構造を適切に定義する必要があります
#[derive(Debug, Clone)]
pub struct MockCamera {
    idx: usize,
    w: u32,
    h: u32,
    is_capture: bool,
    // CameraInfoなどの初期化を追加する必要があります
}

impl CameraInterface for MockCamera {
    fn num_devices() -> usize {
       0
    }
    fn new(idx: usize) -> Self {
        MockCamera {
            idx,
            w: 1912,
            h: 1304,
            is_capture: false,
            // CameraInfoなどの初期化を追加する必要があります
        }
    }
    fn get_info(&self) -> CameraInfo {
        CameraInfo {
            name: "Mock Camera".to_string(),
            idx: self.idx as u32,
            max_width: 1912,
            max_height: 1304,
            supported_img_type: vec![ImgType::RAW8, ImgType::RAW16],
            supported_bins: vec![1, 2, 4, 8],
            is_coolable: false,
        }
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
        // set_roiメソッドの実装
    }

    fn set_img_type(&mut self, img_type: ImgType) {
        // set_img_typeメソッドの実装
    }

    fn get_roi(&self) -> ROIFormat {
        // roiの適当な要素を生成
        ROIFormat {
            startx: 0,
            starty: 0,
            width: self.w,
            height: self.h,
            bin: 1,
            img_type: ImgType::RAW8 as u8,
        }
    }

    fn get_img_type(&self) -> ImgType {
        // imgtypの適当な要素を生成
        ImgType::RAW8
    }

    fn start_capture(&mut self) {
        self.is_capture = true
    }

    fn stop_capture(&mut self) {
        self.is_capture = false
    }

    fn get_frame(&self) -> String {
        let mut rng = rand::thread_rng();
        let buf: Vec<u8> = (0..(self.w * self.h))
            .map(|_| rng.gen_range(0..255))
            .collect();
        println!("=================== get_frame");
        let num = rng.gen_range(0..5) * 100000;
        for i in 0..num{}
        println!("=================== end");

        let buf = base64::encode(buf);
        buf
    }
    fn get_control_value(&self, ctrl_type: ControlType) -> i64 {
        // get_control_valueメソッドの実装
        0
    }
    fn adjust_white_balance(&self) {
        
    }
    fn set_control_value(&self, ctrl_type: ControlType, value: i64, is_auto: i64) {
    }
    fn is_capture(&self) -> bool {
        self.is_capture
    }
    fn set_is_capture(&mut self, is_capture: bool) {
        self.is_capture = is_capture
    }

    fn close(&self) {
        // closeメソッドの実装
    }
}
