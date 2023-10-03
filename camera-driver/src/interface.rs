use serde::{Deserialize, Serialize};

use log::{debug, error, info, warn};
use svbony_camera_rs::libsvb;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ImgType {
    RAW8 = 0,
    RAW10,
    RAW12,
    RAW14,
    RAW16,
    Y8,
    Y10,
    Y12,
    Y14,
    Y16,
    RGB24,
    RGB32,
    END = -1,
}

impl ImgType {
    pub fn from_i32(img_t: &i32) -> ImgType {
        println!("img_type_str: {}", img_t);
        match img_t {
            0 => ImgType::RAW8,
            1 => ImgType::RAW10,
            2 => ImgType::RAW12,
            3 => ImgType::RAW14,
            4 => ImgType::RAW16,
            5 => ImgType::Y8,
            6 => ImgType::Y10,
            7 => ImgType::Y12,
            8 => ImgType::Y14,

            9 => ImgType::Y16,
            10 => ImgType::RGB24,
            11 => ImgType::RGB32,

            _ => ImgType::END,
        }
    }
    pub fn to_svb(img_type: ImgType) -> libsvb::SVB_IMG_TYPE {
        match img_type {
            ImgType::RAW8 => libsvb::SVB_IMG_TYPE_SVB_IMG_RAW8,
            ImgType::RAW10 => libsvb::SVB_IMG_TYPE_SVB_IMG_RAW10,
            ImgType::RAW12 => libsvb::SVB_IMG_TYPE_SVB_IMG_RAW12,
            ImgType::RAW14 => libsvb::SVB_IMG_TYPE_SVB_IMG_RAW14,
            ImgType::RAW16 => libsvb::SVB_IMG_TYPE_SVB_IMG_RAW16,
            ImgType::Y8 => libsvb::SVB_IMG_TYPE_SVB_IMG_Y8,
            ImgType::Y10 => libsvb::SVB_IMG_TYPE_SVB_IMG_Y10,
            ImgType::Y12 => libsvb::SVB_IMG_TYPE_SVB_IMG_Y12,
            ImgType::Y14 => libsvb::SVB_IMG_TYPE_SVB_IMG_Y14,
            ImgType::Y16 => libsvb::SVB_IMG_TYPE_SVB_IMG_Y16,
            ImgType::RGB24 => libsvb::SVB_IMG_TYPE_SVB_IMG_RGB24,
            ImgType::RGB32 => libsvb::SVB_IMG_TYPE_SVB_IMG_RGB32,
            ImgType::END => libsvb::SVB_IMG_TYPE_SVB_IMG_END,
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
    pub fn from_i32(ctrl_idx: &i32) -> ControlType {
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

    pub fn to_svb(ctrl_type: ControlType) -> libsvb::SVB_CONTROL_TYPE {
        let svb_ctrl_t = match ctrl_type {
            ControlType::CONTRAST => libsvb::SVB_CONTROL_TYPE_SVB_CONTRAST,
            ControlType::CURRENT_TEMPERATURE => libsvb::SVB_CONTROL_TYPE_SVB_CURRENT_TEMPERATURE,
            ControlType::COOLER_POWER => libsvb::SVB_CONTROL_TYPE_SVB_COOLER_POWER,
            ControlType::GAMMA_CONTRAST => libsvb::SVB_CONTROL_TYPE_SVB_GAMMA_CONTRAST,
            ControlType::GAIN => libsvb::SVB_CONTROL_TYPE_SVB_GAIN,
            ControlType::GAMMA => libsvb::SVB_CONTROL_TYPE_SVB_GAMMA,
            ControlType::SATURATION => libsvb::SVB_CONTROL_TYPE_SVB_SATURATION,
            ControlType::SHARPNESS => libsvb::SVB_CONTROL_TYPE_SVB_SHARPNESS,
            ControlType::EXPOSURE => libsvb::SVB_CONTROL_TYPE_SVB_EXPOSURE,
            ControlType::WB_R => libsvb::SVB_CONTROL_TYPE_SVB_WB_R,
            ControlType::WB_B => libsvb::SVB_CONTROL_TYPE_SVB_WB_B,
            ControlType::WB_G => libsvb::SVB_CONTROL_TYPE_SVB_WB_G,
            ControlType::FLIP => libsvb::SVB_CONTROL_TYPE_SVB_FLIP,
            ControlType::FRAME_SPEED_MODE => libsvb::SVB_CONTROL_TYPE_SVB_FRAME_SPEED_MODE,
            ControlType::AUTO_TARGET_BRIGHTNESS => {
                libsvb::SVB_CONTROL_TYPE_SVB_AUTO_TARGET_BRIGHTNESS
            }
            ControlType::BLACK_LEVEL => libsvb::SVB_CONTROL_TYPE_SVB_BLACK_LEVEL,
            ControlType::COOLER_ENABLE => libsvb::SVB_CONTROL_TYPE_SVB_COOLER_ENABLE,
            ControlType::TARGET_TEMPERATURE => libsvb::SVB_CONTROL_TYPE_SVB_TARGET_TEMPERATURE,
            ControlType::BAD_PIXEL_CORRECTION_ENABLE => {
                libsvb::SVB_CONTROL_TYPE_SVB_BAD_PIXEL_CORRECTION_ENABLE
            }
        };

        svb_ctrl_t
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInfo {
    pub name: String,
    pub idx: u32,
    pub max_width: u32,
    pub max_height: u32,
    pub supported_img_type: Vec<ImgType>,
    pub supported_bins: Vec<u8>,
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
    fn num_devices() -> usize;
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
    fn start_capture(&mut self);
    fn stop_capture(&mut self);
    fn get_frame(&self) -> String;
    fn get_control_value(&self, ctrl_type: ControlType) -> i64;
    fn set_control_value(&self, ctrl_type: ControlType, value: i64, is_auto: i64);
    fn get_info(&self) -> CameraInfo;
    fn is_capture(&self) -> bool;
    fn set_is_capture(&mut self, is_capture: bool);
    fn adjust_white_balance(&self);
    
    fn close(&self);
}
