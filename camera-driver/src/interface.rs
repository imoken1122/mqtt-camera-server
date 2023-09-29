use serde::{Deserialize, Serialize};

use log::{debug, error, info, warn};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ImgType {
    RAW8 = 0,
    RAW16,
    RGB24,
}
impl ImgType {
    pub fn from_u8(img_type_str: u8) -> ImgType {
        match img_type_str {
            0 => ImgType::RAW8,
            1 => ImgType::RAW16,
            2 => ImgType::RGB24,
            _ => {
                panic!("Unknown image type");
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ControlType {
    GAIN = 0,
    EXPOSURE,
    GAMMA,
    GAMMA_CONTRAST,
    WB_R,
    WB_G,
    WB_B,
    FLIP,
    FRAME_SPEED_MODE,
    CONTRAST,
    SHARPNESS,
    SATURATION,
    AUTO_TARGET_BRIGHTNESS,
    BLACK_LEVEL,
    COOLER_ENABLE,
    TARGET_TEMPERATURE,
    CURRENT_TEMPERATURE,
    COOLER_POWER,
    BAD_PIXEL_CORRECTION_ENABLE,
}
impl ControlType {
    pub fn from_u8(ctrl_idx: &u8) -> ControlType {
        match ctrl_idx {
            0 => ControlType::GAIN,
            1 => ControlType::EXPOSURE,
            2 => ControlType::GAMMA,
            3 => ControlType::GAMMA_CONTRAST,
            4 => ControlType::WB_R,
            5 => ControlType::WB_G,
            6 => ControlType::WB_B,
            7 => ControlType::FLIP,
            8 => ControlType::FRAME_SPEED_MODE,
            9 => ControlType::CONTRAST,
            10 => ControlType::SHARPNESS,
            11 => ControlType::SATURATION,
            12 => ControlType::AUTO_TARGET_BRIGHTNESS,
            13 => ControlType::BLACK_LEVEL,
            14 => ControlType::COOLER_ENABLE,
            15 => ControlType::TARGET_TEMPERATURE,
            16 => ControlType::CURRENT_TEMPERATURE,
            17 => ControlType::COOLER_POWER,
            18 => ControlType::BAD_PIXEL_CORRECTION_ENABLE,
            _ => {
                panic!("Unknown control type");
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInfo {
    pub name: String,
    pub idx: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub supported_img_type: Vec<ImgType>,
    pub supported_bins: Vec<i32>,
    pub is_coolable: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ROIFormat {
    pub startx: u32,
    pub starty: u32,
    pub width: u32,
    pub height: u32,
    pub bin: u8,
    pub img_type: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlCaps {
    pub name: String,
    pub max_value: u64,
    pub min_value: u64,
    pub default_value: u64,
    pub is_auto_supported: bool,
    pub is_writable: bool,
    pub control_type: ControlType,
}

pub trait CameraInterface {
    fn new(idx: usize) -> Self;
    fn set_roi(
        &mut self,
        startx: u32,
        starty: u32,
        width: u32,
        height: u32,
        bin: u8,
        img_type: ImgType,
    );
    fn set_img_type(&mut self, img_type: ImgType);
    fn get_roi(&self) -> ROIFormat;
    fn get_img_type(&self) -> ImgType;
    fn start_capture(&self);
    fn stop_capture(&self);
    fn get_frame(&self) -> Vec<u8>;
    fn get_control_value(&self, ctrl_type: ControlType) -> i64;
    fn set_control_value(&self, ctrl_type: ControlType, value: i64, is_auto: i32);
    fn get_info(&self) -> CameraInfo;

    fn close(&self);
}
