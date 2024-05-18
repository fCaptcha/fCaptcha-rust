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
    static ref HTTP_CLIENT: Client = ClientBuilder::new().danger_accept_invalid_certs(true).timeout(Duration::from_secs(25)).build().unwrap();
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

fn create_headers(headers: &HeaderMap) -> DortCapResult<HeaderMap> {
    let mut headers = headers.clone();
    headers.insert("Accept", HeaderValue::try_from("image/png")?);
    headers.insert("Accept-Encoding", HeaderValue::try_from("gzip, deflate, br")?);
    headers.insert("Accept-Language", HeaderValue::try_from("en-US,en;q=0.7")?);
    headers.insert("Origin", HeaderValue::try_from("https://client-api.arkoselabs.com")?);
    headers.insert("Referer", HeaderValue::try_from("https://client-api.arkoselabs.com/fc/assets/ec-game-core/game-core/1.17.1/standard/index.html")?);
    headers.insert("Sec-Fetch-Dest", HeaderValue::try_from("empty")?);
    headers.insert("Sec-Fetch-Mode", HeaderValue::try_from("cors")?);
    headers.insert("Sec-Fetch-Site", HeaderValue::try_from("same-origin")?);
    headers.insert("User-Agent", HeaderValue::try_from(format!("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:124.0) Gecko/20100101 Firefox/{}.0", fastrand::u16(1..2414)))?);
    return Ok(headers);
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
        let raw_grid_bytes = http_session.get(image_url).headers(create_headers(headers)?).send().await?.bytes().await?;
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
        if game_type == 3 {
            let mut tasks: Vec<JoinHandle<DortCapResult<TaskResult>>> = Vec::new();
            for tile in &tiles {
                let hash = tile.hash.clone();
                let key = format!("Game Type {}:Tile Hashes ({}x{}):{}:{}", game_type, tile.width, tile.height, variant, &*hash);
                tasks.push(tokio::spawn(async move {
                    let idx = idx;
                    Ok(TaskResult::new(idx, IMAGE_DATABASE.get().await.clone().get(key).await, hash))
                }));
                idx += 1;
            }
            for task in tasks {
                let tile = task.await??;
                match tile.result {
                    Ok(res) => {
                        match &*res {
                            "good" => {
                                if ARGUMENTS.print_colliding_hashes {
                                    println!("good {game_type} {variant}");
                                }
                                selected_tile = tiles.clone().iter().position(|x| x.hash.eq(tile.hash.as_str())).ok_or(CodeErr(0x01, "TILE_SELECTION"))? as u8;
                                found = true;
                            },
                            "bad" => {
                                if ARGUMENTS.print_colliding_hashes {
                                    println!("bad {game_type} {variant}");
                                }
                                indexes.retain(|x| *x as usize != tiles.clone().iter().position(|x| x.hash.eq(tile.hash.as_str())).unwrap());
                            }
                            _ => {},
                        }
                    }
                    _ => {}
                }
            }
        } else {
            let instruction_tile = instruction_tile.as_ref();
            // selected_tile = tiles.clone().iter().position(|x| x.hash.eq(&*tile.hash)).ok_or(CodeErr(0x01, "TILE_SELECTION"))? as u8;
            let mut tasks: Vec<JoinHandle<Result<(i32, String), DortCapError>>> = Vec::new();
            let mut check_tasks: Vec<JoinHandle<Result<bool, DortCapError>>> = Vec::new();
            if let Some(instruction_tile) = instruction_tile {
                for tile in &tiles {
                    let hash = tile.hash.clone();
                    let key = format!("Game Type {}:Tile Hashes ({}x{}):{}:{}", game_type, tile.width, tile.height, variant, &*hash);
                    tasks.push(tokio::spawn(async move {
                        let mut db = IMAGE_DATABASE.get().await.clone();
                        let idx = idx;
                        if !CACHE.contains_async(&*hash).await {
                            let result: RedisResult<Vec<String>> = db.lrange(&key, 0, -1).await;
                            if let Ok(result) = result {
                                let _ = CACHE.insert_async(hash.clone(), RwLock::with_max_readers(result, 1048576)).await;
                            } else {
                                let _ = CACHE.insert_async(hash.clone(), RwLock::with_max_readers(Vec::default(), 1048576)).await;
                            }
                        }
                        Ok((idx, hash))
                    }));
                    idx += 1;
                }
                for task in tasks {
                    let tile = task.await??;
                    let h = tile.1;
                    let rwl = CACHE.get_async(&*h).await.ok_or(CodeErr(0x01, "HM_GET_CACHE"))?;
                    let rwl2 = rwl.get().read().await;
                    for instr in &*rwl2 {
                        let cloned_instruction_hash = instr.clone();
                        let cloned_tile = instruction_tile.clone();
                        let variant_string = variant.to_owned();
                        check_tasks.push(tokio::spawn(async move {
                            // would avoid using an Option<T> but the error returned is a fucking enum and I couldn't care less.
                            let dist = cloned_tile.hash_raw.dist(&ImageHash::from_base64(&*cloned_instruction_hash).ok().ok_or(CodeErr(0x01, "HAMMING_DISTANCE"))?);
                            if dist <= 0 {
                                if ARGUMENTS.print_colliding_hashes {
                                    println!("good {game_type} {variant_string}");
                                }
                                return Ok(true);
                            }
                            Ok(false) // so fucking aids but works... :/
                        }));
                    }
                    drop(rwl2);
                    while let Some(t) = check_tasks.pop() {
                        let t = t.await??;
                        if t {
                            found = true;
                            selected_tile = tiles.clone().iter().position(|x| x.hash.eq(&*h)).ok_or(CodeErr(0x01, "TILE_SELECTION"))? as u8;
                            break;
                        }
                    }
                }
            }
        }
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