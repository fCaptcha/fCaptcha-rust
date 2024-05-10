pub mod structs;

use self::{
    structs::check_captcha::CheckCaptchaMotionData,
    structs::get_captcha::GetCaptchaMotionData
};

pub struct MotionDataGenerator {
    pub get_captcha_motion_data: Option<GetCaptchaMotionData>,
    pub check_captcha_motion_data: Option<CheckCaptchaMotionData>
}

impl MotionDataGenerator {
    pub fn new() -> MotionDataGenerator {
        Self {
            get_captcha_motion_data: None,
            check_captcha_motion_data: None,
        }
    }
}