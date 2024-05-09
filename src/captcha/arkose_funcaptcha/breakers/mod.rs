use reqwest::Client;
use reqwest::header::HeaderMap;
use serde_json::{Value, json};
use crate::commons::error::DortCapError::CodeErr;
use crate::commons::error::DortCapResult;

pub mod jshelper;


pub(crate) async fn get_answers(headers: HeaderMap, client: &Client, dapi_script: &str, game_type: i32, lw: &Vec<Value>, li: &str) -> DortCapResult<Vec<Value>> {
    if game_type == 3 || dapi_script.eq("NOT_REQUIRED") {
        return Ok(Vec::new());
    }
    let d0: Vec<&str> = li.split('.').collect();
    let d1 = d0[0];
    let d2 = d0[1];
    let new_val_vec: Vec<Value> = lw.iter().map(|item| {
        let mut result = merge(json!({}), item);
        let new_map = json!({ d1: d2 });
        result = merge(result.to_owned(), &new_map);
        return result;
    }).collect();
    let dapib_code = client.get(dapi_script).headers(headers).send().await?.text().await?;
    Ok(jshelper::breakers(&*dapib_code, json!(new_val_vec))?.as_array().ok_or(CodeErr(0x01, "TG_API_BREAKERS"))?.to_vec())
}

fn merge(mut lw: Value, li: &Value) -> Value {
    if let (Value::Object(mut lw_map), Value::Object(li_map)) = (lw.clone(), li) {
        for (key, value) in li_map {
            lw_map.insert(key.clone(), value.clone());
        }
        lw = Value::from(lw_map);
    }
    lw
}
