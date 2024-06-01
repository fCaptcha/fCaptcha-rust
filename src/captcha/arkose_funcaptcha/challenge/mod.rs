use crate::DortCapResult;
use scc::HashMap;
use std::str::FromStr;
use std::time::Duration;
use async_once::AsyncOnce;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use fastrand::choice;
use image::{DynamicImage, EncodableLayout, load_from_memory};
use image_hasher::ImageHash;
use lazy_static::lazy_static;
use random_string::generate;
use serde_json::{json, Value};
use reqwest::{Client, ClientBuilder};
use redis::{AsyncCommands, RedisResult};
use reqwest::header::{HeaderMap, HeaderValue};
use rocket::figment::Source::Code;
use rocket::form::validate::len;
use tokio::sync::{RwLock, RwLockReadGuard};
use tokio::task::{JoinHandle, spawn_blocking};
use tokio::time::sleep;
use warp::query::raw;
use self::tile::Tile;
use super::encryption::cryptojs_decrypt;
use crate::{ARGUMENTS, DORTCAP_CONFIG, IMAGE_DATABASE, XEvilNode};
use crate::captcha::arkose_funcaptcha::imageprocessing;
use crate::captcha::arkose_funcaptcha::imageprocessing::{process_dynamic_image};
use crate::commons::error::DortCapError;
use crate::commons::error::DortCapError::CodeErr;
use crate::commons::RUNTIME;

pub mod tile;

lazy_static! {
    static ref HTTP_CLIENT: Client = ClientBuilder::new().timeout(Duration::from_secs(25)).build().unwrap();
    static ref CACHE: HashMap<String, RwLock<Vec<String>>> = HashMap::default();
    static ref CAPBYPASS_KEY: &'static str = "CB-9483a6f6b48c45e7aeba59417acf8fbc";
}

async fn solve_cb(image_b64: &str, variant: &str) -> DortCapResult<u8> {
    let url = "https://capbypass.com/api/createTask";
    let payload = json!({
        "clientKey": &*CAPBYPASS_KEY,
        "task": {
            "type": "FunCaptchaClassification",
            "image": image_b64,
            "question": variant
        }
    });
    let mut task_res: Value = HTTP_CLIENT.post(url).json(&payload).send().await?.json().await?;;
    while task_res["errorMessage"].eq("red lock acquired") {
        task_res = HTTP_CLIENT.post(url).json(&payload).send().await?.json().await?;
    }
    for _ in 0..150 {
        sleep(Duration::from_millis(50)).await;
        if task_res["errorId"] == 0 {
            let url = "https://capbypass.com/api/getTaskResult";
            let payload = json!({
                "clientKey": &*CAPBYPASS_KEY,
                "taskId": task_res["taskId"]
            });
            let task_res: Value = HTTP_CLIENT.post(url).json(&payload).send().await?.json().await?;
            if task_res["solution"].is_string() {
                if let Ok(ans) = u8::from_str(task_res["solution"].as_str().ok_or(CodeErr(0x01, "FAILED_TO_SOLVE"))?) {
                    return Ok(ans);
                }
            }
        }
    }
    Err(CodeErr(0x01, "SOLVE_FAILED"))
}

pub async fn get_answer(xevil_node: &XEvilNode, difficulty: u8, variant: &str, raw_grid: &Vec<u8>) -> DortCapResult<u8> {
    let lock = xevil_node.queue_lock.write().await;
    while *xevil_node.current_queue_size.read().await > xevil_node.queue_size {}
    drop(lock);
    *xevil_node.current_queue_size.write().await += 1;
    let mut post_data = std::collections::HashMap::new();
    let host = &*xevil_node.host;
    let port = xevil_node.port;
    let key = &*xevil_node.api_key;
    post_data.insert("key", String::from(key));
    post_data.insert("method", String::from("image"));
    post_data.insert("imginstructions", String::from(variant));
    post_data.insert("body", BASE64_STANDARD.encode(raw_grid));
    post_data.insert("json", String::from("1"));
    let response_from_server: Value = serde_json::from_str(&HTTP_CLIENT.post(format!("http://{}:{}/in.php", host, port)).form(&post_data).send().await?.text().await?)?;
    if !response_from_server["error"].is_string() {
        let request_id = &response_from_server["request"];
        if request_id.is_string() {
            let id = request_id.as_str().unwrap();
            let response_from_server: Value = serde_json::from_str(&HTTP_CLIENT.get(format!("http://{}:{}/res.php?action=get&id={}&json=1", host, port, id)).send().await?.text().await?)?;
            if !response_from_server["error"].is_string() {
                let answer = u8::from_str(response_from_server["request"].as_str().unwrap())?;
                return Ok(answer - 1);
            }
        }
    }
    Ok(fastrand::u8(0..difficulty))
}

#[derive(Clone)]
pub struct Challenge {
    pub instruction_tile: Option<Tile>,
    pub tiles: Vec<Tile>,
    pub grid_size: u32,
    pub selected_tile: u8,
    pub difficulty: u8,
    pub variant: String,
    pub game_type: u8,
    pub raw_grid: Vec<u8>
}


struct TaskResult {
    index: i32,
    result: RedisResult<String>,
    hash: String
}

impl TaskResult {
    pub fn new(index: i32, result: RedisResult<String>, hash: String) -> Self {
        return TaskResult {
            index,
            result,
            hash,
        }
    }
}


impl Challenge {
    pub async fn new(http_session: &Client, headers: &HeaderMap, difficulty: u8, game_type: u8, variant: &str, image_url: &str, decryption_key: &Option<String>) -> DortCapResult<Self> {
        let raw_grid_bytes = http_session.get(image_url).headers(headers.clone()).send().await?.bytes().await?;
        let mut raw_grid = raw_grid_bytes.to_vec();
        if decryption_key.is_some() {
            raw_grid = BASE64_STANDARD.decode(cryptojs_decrypt(&String::from_utf8(raw_grid)?, decryption_key.as_ref().unwrap())?)?;
        }
        let image = load_from_memory(raw_grid.as_slice())?;
        let mut tiles = Vec::new();
        let grid_size = image.height() / 2;
        let mut instruction_tile: Option<Tile> = None;
        if game_type == 4 {
            let instruction_image = image.crop_imm(0, 200, 130, 400);
            instruction_tile = Some(Tile::new(&instruction_image, true).await?);
            for tile in 0..difficulty as u32 {
                let tile_image = image.crop_imm(grid_size * tile, 0, 200, 200);
                tiles.push(Tile::new(&tile_image, false).await?);
            }
        } else if game_type == 3 {
            for y in 0..2 {
                for x in 0..3 {
                    let tile_image = image.crop_imm(x, y, grid_size, grid_size);
                    tiles.push(Tile::new(&tile_image, false).await?);
                }
            }
        }
        let mut selected_tile = fastrand::u8(0..difficulty);
        let mut found = false;
        let mut indexes: Vec<u8> = (0..difficulty).collect();
        let mut idx = 0;
        selected_tile = *choice(&indexes).unwrap_or(&selected_tile);
        if !found && indexes.len() != 1 {
            match &*ARGUMENTS.ai_fallback_type.to_ascii_lowercase() {
                "xevil" => {
                    let mut node = choice(&DORTCAP_CONFIG.solving.xevil_nodes).ok_or(CodeErr(0x01, "IMAGE_SOLVER_NODE"))?;
                    while *node.current_queue_size.read().await > node.queue_size {
                        node = choice(&DORTCAP_CONFIG.solving.xevil_nodes).ok_or(CodeErr(0x02, "IMAGE_SOLVER_NODE"))?;
                    }
                    if let Ok(selected_tile_result) = get_answer(node, difficulty, variant, &raw_grid).await {
                        selected_tile = selected_tile_result;
                    }
                    *node.current_queue_size.write().await -= 1;
                }
                "cb" => {
                    // let task = FunCaptchaClassificationTask::new(&*BASE64_STANDARD.encode(raw_grid.as_slice()), "orbit_match_game");
                    // let t = CAPBYPASS.create_and_wait(task).await?;
                    // selected_tile = u8::from_str(&*t)?;
                    if let Ok(answer) = solve_cb(&*BASE64_STANDARD.encode(raw_grid.as_slice()), variant).await {
                        selected_tile = answer;
                    }
                }
                _ => {}
            }
        }
        Ok(Challenge {
            instruction_tile,
            tiles,
            grid_size,
            selected_tile,
            difficulty,
            game_type,
            raw_grid,
            variant: String::from(variant)
        })
    }

    pub async fn save_tiles(&self) -> DortCapResult<()> {
        let mut index = 0;
        for tile in &self.tiles {
            let mut database = IMAGE_DATABASE.get().await.to_owned();
            let opt = self.instruction_tile.as_ref();
            if let Some(instruction_tile) = opt {
                let hash = &*tile.hash;
                if index == self.selected_tile {
                    let mut secondary = CACHE.get_async(hash).await.ok_or(CodeErr(0x01, "HM_SET_REDIS"))?;
                    let secondary = secondary.get();
                    let mut secondary_guard = secondary.write().await;
                    if !secondary_guard.contains(&instruction_tile.hash) {
                        secondary_guard.push(String::from(&*instruction_tile.hash));
                        loop {
                            let result: RedisResult<()> = database.lpush(format!("Game Type {}:Tile Hashes ({}x{}):{}:{}", self.game_type, tile.width, tile.height, self.variant, hash), &*instruction_tile.hash).await;
                            if result.is_ok() {
                                break;
                            }
                        }
                    }
                    drop(secondary_guard);
                }
            } else {
                loop {
                    let hash = tile.hash.clone();
                    let result: RedisResult<String> = database.set(format!("Game Type {}:Tile Hashes ({}x{}):{}:{}", self.game_type, tile.width, tile.height, self.variant, hash), if index == self.selected_tile { "good" } else { "bad" }).await;
                    if result.is_ok() {
                        break;
                    }
                }
            }
            index += 1;
        }
        Ok(())
    }
}