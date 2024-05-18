#![recursion_limit = "512"]

mod commons;
mod captcha;
mod tools;
mod api;

use std::borrow::Cow;
use std::fs::read_to_string;
use std::collections::HashMap;
use std::f64;
use std::fmt::format;
use std::io::{stdout};
use std::str::FromStr;
use std::time::Duration;
use crate::captcha::arkose_funcaptcha::structs::{FunCaptchaRequest};
use clap::Parser;
use crossterm::{execute};
use serde::{Deserialize, Serialize};
use crossterm::terminal::SetTitle;
use lazy_static::lazy_static;
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};
use rocket::http::Status;
use rocket::serde::json::Json;
use tokio::sync::{RwLock};
use v8::{new_unprotected_default_platform, V8};
use async_once::AsyncOnce;
use fastrand::u16;
use fmtools::obfstr;
use random_string::generate;
use rocket::response::status::Custom;
use rocket::{get, post, routes};
use serde_json::{json, to_string_pretty, Value};
use roblox_register::register_roblox;
use crate::api::customer_api::{get_balance, reduce_bal, topup};
use crate::captcha::arkose_funcaptcha::ArkoseSession;
use crate::captcha::arkose_funcaptcha::bda::templates::BDATemplate;
use crate::captcha::arkose_funcaptcha::encryption::murmur;
use crate::captcha::recaptcha_v3::ReCaptchaV3;
use crate::commons::console::{created_account, solved};
use crate::commons::console::SolveType::{CUSTOMER, INTERNAL};
use crate::commons::error::{DortCapError, DortCapResult};
use crate::commons::RUNTIME;
use crate::tools::generators::github::fetch_blob;
use crate::tools::generators::roblox_register;
use crate::tools::generators::outlook::OutlookCreator;
use crate::tools::generators::roblox_login::login_roblox;

#[derive(Serialize, Deserialize)]
pub struct XEvilNode {
    #[serde(skip, default = "RwLock::default")]
    pub current_queue_size: RwLock<u32>,
    pub queue_size: u32,
    #[serde(skip, default = "RwLock::default")]
    pub queue_lock: RwLock<()>,
    pub api_key: String,
    pub host: String,
    pub port: u16,
}

#[derive(Serialize, Deserialize)]
pub struct NetConfig {
    pub proxy: String,
    pub subnet: String
}

#[derive(Serialize, Deserialize)]
pub struct SolvingConfig {
    xevil_nodes: Vec<XEvilNode>,
}


#[derive(Serialize, Deserialize)]
pub struct HashingConfig {
    hash_size: u32,
}


#[derive(Serialize, Deserialize)]
pub struct DortCapConfig {
    solving: SolvingConfig,
    networking: NetConfig,
    hashing: HashingConfig,
}

#[derive(Parser)]
#[command(author = "Dort", version = "2.1.0", about = "DortCap Captcha Solver")]
struct Args {
    /// Number of threads to use, spread equally across each task.
    #[arg(short, long, default_value_t = 10, value_name = "threads")]
    threads: usize,
    /// Attempt to request an audio challenge (falls back to image on fail)
    #[arg(short = 'a', long = "try-audio", default_value_t = false)]
    try_audio: bool,
    /// Print unsuccessful CAPTCHA results.
    #[arg(short = 'p', long, default_value_t = false)]
    print_bad_captcha_results: bool,
    /// Print found image collisions and if it's good or bad.
    #[arg(short = 'P', long, default_value_t = false)]
    print_colliding_hashes: bool,
    /// Use AI to recognise images (if using XEvil, it must be put in DortCap.toml, while running and authenticated)
    #[arg(long, short = 'A', value_name = "ai_fb_type", default_value_t = String::from("NO_FALLBACK"), default_values = vec ! ["XEVIL", "DCT0", "DCT1", "NO_FALLBACK"])]
    ai_fallback_type: String,
    /// Start API Server.
    #[arg(short, long, default_value_t = false)]
    start_api: bool,
    /// hehe :3
    #[arg(short = 'D', long, default_value_t = false)]
    discord: bool,
    /// Mass create Roblox Accounts.
    #[arg(short, long, default_value_t = false)]
    roblox: bool,
    /// Sets max synchronous tasks. 1 task = stable ~75 threads
    #[arg(short, long, default_value_t = 64, value_name = "max_tasks")]
    max_sync_tasks: usize,
    /// Create Outlook accounts.
    #[arg(short = 'o', long, default_value_t = false)]
    outlook: bool,
    /// Site key to use (defaults to Twitter Desktop).
    #[arg(short, long, default_value_t = String::from("2CB16598-CB82-4CF7-B332-5990DB66F3AB"))]
    key: String,
    /// Outlook name prefix (--outlook only)
    #[arg(short = 'O', long, default_value_t = String::from("dc_"))]
    name_prefix: String,
    /// Proxy type to use.
    #[arg(long, default_value_t = String::from("socks5h"))]
    proxy_type: String,
    /// Test roblox.com login Site Key for bad CAPTCHA results
    #[arg(long, default_value_t = false)]
    test_roblox_login: bool,
    /// Max AI Queue size, better to leave this under 185 if using XEvil.
    #[arg(long, default_value_t = 200, value_name = "max_queue_size")]
    max_queue_size: usize,
}

async fn get_redis_instance(db_num: u16) -> ConnectionManager {
    loop {
        let client = Client::open(format!("redis://default:ACCA5B570561DCFA5ACB1417C69F2900DAFF8A4FD39A2E66C36DF2BD796F0BE1CFEA8AF2DB18153874215E08BFDEC4A89A397EC53E52DAC33A1E9D0B17A52D43@45.45.238.213:42081/{db_num}"));
        if client.is_ok() {
            loop {
                let connection_manager = client.as_ref().unwrap().get_connection_manager().await;
                if connection_manager.is_ok() {
                    return connection_manager.unwrap();
                }
            }
        }
    }
}

#[get("/stats.json")]
async fn dortcap_version() -> Custom<String> {
    let json_data = json!({
        "version": "1.3.0",
        "branch": "release-candidate-5",
        "solvers": {
            "arkose": {
                "capi-support": "any"
            },
            "hcaptcha": {

            },
            "threads": *THREADS.read().await
        },
        "authors": [
            "dort", // I made the entire solver pretty much LOL
            "slotth"
        ]
    });
    return Custom(Status::Ok, to_string_pretty(&json_data).unwrap());
}

lazy_static! {
    static ref PROXIES: RwLock<Vec<String>> = (|| {
        let data = read_to_string("data/proxies.txt").unwrap();
        let data = data.split("\n").map(|s| s.replace("\r", ""));
        RwLock::new(data.collect())
    })();
    static ref DORTCAP_CONFIG: DortCapConfig = toml::from_str(&*read_to_string("data/DortCap.toml").expect("Config file not found in data/DortCap.toml!")).expect("Config parse failure.");
    static ref THREADS: RwLock<HashMap<String, i32>> = RwLock::default();
    static ref SOLVED: RwLock<u128> = RwLock::default();
    static ref ARGUMENTS: Args = Args::parse();
    static ref REDIS_USERS_PPU: AsyncOnce<ConnectionManager> = AsyncOnce::new(async {
        return get_redis_instance(300).await;
    });
    static ref REDIS_USERS: AsyncOnce<ConnectionManager> = AsyncOnce::new(async {
        return get_redis_instance(301).await;
    });
    static ref FINGERPRINTS: AsyncOnce<ConnectionManager> = AsyncOnce::new(async {
        return get_redis_instance(302).await;
    });
    static ref IMAGE_DATABASE: AsyncOnce<ConnectionManager> = AsyncOnce::new(async {
        return get_redis_instance(303).await;
    });
}

#[tokio::main(flavor = "multi_thread", worker_threads = 64)]
async fn main() -> DortCapResult<()> {
    let platform = new_unprotected_default_platform(600, false).make_shared();
    V8::initialize_platform(platform);
    V8::initialize();
    RUNTIME.spawn(async {
        loop {
            let current_customer_threads: i32 = THREADS.read().await.values().sum();
            let solved = *SOLVED.read().await;
            if execute!(stdout(), SetTitle(format!("ByeCaptcha Solver | Customer Threads: {current_customer_threads} | Solved Captchas: {solved}"))).is_ok() {
                tokio::time::sleep(Duration::from_millis(350)).await;
            }
        }
    });
    RUNTIME.spawn(async move {
        let _rocket = rocket::build()
            .mount("/solver", routes![dortcap_version, arkose_solve, get_balance])
            .mount("/wh", routes![topup])
            .launch()
            .await.expect("API Crashed.");
    });
    for _ in 0..ARGUMENTS.threads {
        RUNTIME.spawn(async move {
            loop {
                if ARGUMENTS.roblox {
                    register_roblox().await;
                    continue;
                }
                if ARGUMENTS.test_roblox_login {
                    login_roblox().await;
                    continue;
                }
                if ARGUMENTS.outlook {
                    if let Some(outlook) = OutlookCreator::new().await {
                        if let Ok(account) = outlook.create_account().await {
                            created_account(Some(&*account));
                        }
                    }
                    continue;
                }
                let ses = ArkoseSession::new(FunCaptchaRequest {
                    site_url: String::from("https://iframe.arkoselabs.com"),
                    site_key: String::from(&*ARGUMENTS.key),
                    bda_template: BDATemplate {
                        document_referrer: Some(String::from("https://iframe.arkoselabs.com/")),
                        window_ancestor_origins: Some(vec![
                            String::from("https://iframe.arkoselabs.com"),
                            String::from("https://iframe.arkoselabs.com")
                        ]),
                        window_tree_index: Some(vec![1, 0]),
                        window_tree_structure: Some(String::from("[[],[[]]]")),
                        window_location_href: Some(String::from("https://client-api.arkoselabs.com/v2/2.4.5/enforcement.6c9d6e9be9aa044cc5ce9548b4abe1b0.html")),
                        client_config_sitedata_location_href: Some(String::from("https://iframe.arkoselabs.com/2CB16598-CB82-4CF7-B332-5990DB66F3AB/index.html")),
                        client_config_surl: Some(String::from("https://client-api.arkoselabs.com")),
                        client_config_language: None
                    },
                    audio: false,
                    data: fetch_blob().await,
                    proxy: None,
                    arkose_api_url: String::from("https://client-api.arkoselabs.com"),
                }).await;
                if let Ok(ref session) = ses {
                    let sr = session.solve().await;
                    if let Ok(ref solve_result) = sr {
                        solved(INTERNAL, solve_result.token.as_deref(), solve_result.variant.as_deref(), solve_result.waves.as_ref(), solve_result.solved.as_ref()).await;
                    }
                }
            }
        });
    }
    println!("Post API Start.");
    loop {}
}

async fn failed_authentication(key: &str) -> Option<()> {
    if THREADS.read().await.contains_key(key) {
        return Some(());
    }
    let mut redis_users = REDIS_USERS_PPU.get().await.clone();
    let user: HashMap<String, String> = redis_users.hgetall(key.to_string()).await.ok()?;
    let balance = &**user.get("balance")?;
    if f64::from_str(balance).ok()? <= 0.0 {
        return None;
    }
    Some(())
}

pub fn default_proxy() -> Option<String> {
    Some(DORTCAP_CONFIG.networking.proxy.replace("%SESSION_ID%", &*generate(23, "abcdef1234567890")))
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct ArkoseRequest {
    #[serde(default = "default_proxy")]
    proxy: Option<String>,
    blob: Option<String>,
    surl: Option<String>,
    site_url: String,
    site_key: String,
    api_key: String,
    bda_template: Option<BDATemplate>
}

fn err_fn(err: DortCapError) -> Custom<Json<Value>> {
    Custom(Status::InternalServerError, Json(json!({
        "error": err.to_string()
    })))
}


#[post("/arkose", format = "json", data = "<data>")]
async fn arkose_solve(data: Json<ArkoseRequest>) -> Result<Custom<Json<Value>>, Custom<Json<Value>>> {
    if failed_authentication(&*data.api_key).await.is_none() {
        return Ok(Custom(Status::Unauthorized, Json(json!({
            "error": "INVALID_KEY_OR_THREAD_LIMIT_REACHED"
        }))));
    }
    let ark_url = data.surl.as_deref().unwrap_or("https://client-api.arkoselabs.com");
    let mut bda_template = BDATemplate {
        document_referrer: None,
        window_ancestor_origins: None,
        window_tree_index: None,
        window_tree_structure: None,
        window_location_href: None,
        client_config_sitedata_location_href: None,
        client_config_surl: None,
        client_config_language: None,
    };
    if data.site_key.eq("B7D8911C-5CC8-A9A3-35B0-554ACEE604DA") {
        bda_template = BDATemplate {
            document_referrer: Some(String::from("https://iframe.arkoselabs.com/")),
            window_ancestor_origins: Some(vec![
                String::from("https://signup.live.com"),
                String::from("https://iframe.arkoselabs.com"),
            ]),
            window_tree_index: Some(vec![
                1,
                0
            ]),
            window_tree_structure: Some(String::from("[[[]],[[]]]")),
            window_location_href: None,
            client_config_sitedata_location_href: Some(String::from("https://iframe.arkoselabs.com/B7D8911C-5CC8-A9A3-35B0-554ACEE604DA/index.html")),
            client_config_surl: Some(String::from("https://client-api.arkoselabs.com")),
            client_config_language: Some(String::from("en"))
        };
    } else if data.site_key.eq("476068BF-9607-4799-B53D-966BE98E2B81") {
        bda_template = BDATemplate {
            document_referrer: Some(String::from("https://www.roblox.com/")),
            window_ancestor_origins: Some(vec![
                String::from("https://www.roblox.com"),
                String::from("https://www.roblox.com")
            ]),
            window_tree_index: Some(vec![1, 0]),
            window_tree_structure: Some(String::from("[[],[[]]]")),
            window_location_href: None,
            client_config_sitedata_location_href: Some(String::from("https://www.roblox.com/arkose/iframe")),
            client_config_surl: Some(String::from("https://roblox-api.arkoselabs.com")),
            client_config_language: None,
        };
    } else if data.site_key.eq("A2A14B1D-1AF3-C791-9BBC-EE33CC7A0A6F") {
        bda_template = BDATemplate {
            document_referrer: Some(String::from("https://www.roblox.com/")),
            window_ancestor_origins: Some(vec![
                String::from("https://www.roblox.com/"),
                String::from("https://www.roblox.com/")
            ]),
            window_tree_index: Some(vec![0, 0]),
            window_tree_structure: Some(String::from("[[[]]]")),
            window_location_href: None,
            client_config_sitedata_location_href: Some(String::from("https://www.roblox.com/arkose/iframe")),
            client_config_surl: Some(String::from("https://roblox-api.arkoselabs.com")),
            client_config_language: None,
        };
    } else if data.site_key.eq("867D55F2-24FD-4C56-AB6D-589EDAF5E7C5") {
        bda_template = BDATemplate {
            document_referrer: Some(String::from("https://iframe.arkoselabs.com/")),
            window_ancestor_origins: Some(vec![
                String::from("https://iframe.arkoselabs.com"),
                String::from("https://iframe.arkoselabs.com")
            ]),
            window_tree_index: Some(vec![1, 0]),
            window_tree_structure: Some(String::from("[[],[[]]]")),
            window_location_href: Some(String::from("https://client-api.arkoselabs.com/v2/2.4.5/enforcement.6c9d6e9be9aa044cc5ce9548b4abe1b0.html")),
            client_config_sitedata_location_href: Some(String::from("https://iframe.arkoselabs.com/2CB16598-CB82-4CF7-B332-5990DB66F3AB/index.html")),
            client_config_surl: Some(String::from("https://client-api.arkoselabs.com")),
            client_config_language: None
        }
    }
    let arkose = ArkoseSession::new(FunCaptchaRequest {
        arkose_api_url: String::from(ark_url),
        audio: false,
        bda_template: data.bda_template.clone().unwrap_or(bda_template),
        data: data.blob.clone(),
        site_key: data.site_key.clone(),
        proxy: data.proxy.clone(),
        site_url: data.site_url.clone(),
    }).await.map_err(err_fn)?;
    let result = arkose.solve().await.map_err(err_fn)?;
    solved(CUSTOMER, result.token.as_deref(), result.variant.as_deref(), result.waves.as_ref(), result.solved.as_ref()).await;
    if let Some(solved) = result.solved.as_ref() {
        if *solved {
            reduce_bal(&*data.api_key, 0.00025).await;
        }
    }
    return Ok(Custom(Status::Ok, Json(json!({
        "solved": result.solved,
        "token": result.token,
        "game": {
            "variant": result.variant,
            "waves": result.waves
        }
    }))));
}

