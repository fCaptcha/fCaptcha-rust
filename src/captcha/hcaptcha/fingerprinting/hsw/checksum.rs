use crc32fast::hash as calculate_checksum;
use serde_json::{from_str, Value};
use crate::commons::error::FCaptchaResult;
use crate::conv_option;

pub fn finalize_payload(input_payload: &str) -> FCaptchaResult<String> {
    let mut fp: Value = from_str(input_payload)?;
    let as_array = conv_option!(fp["rand"].as_array_mut())?;
    let scale = 2.3283064365386963e-10f64;
    let mut checksum = calculate_checksum(input_payload.as_ref()) as f64;
    checksum *= scale;
    as_array.push(Value::from(checksum));
    Ok(fp.to_string())
}