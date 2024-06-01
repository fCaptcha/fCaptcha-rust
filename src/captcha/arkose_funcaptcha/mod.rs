use std::collections::{BTreeMap, HashMap};
use std::num::NonZeroU16;
use std::sync::Arc;
use percent_encoding::{utf8_percent_encode, percent_decode, AsciiSet, CONTROLS};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use async_once::AsyncOnce;
use fastrand::{i32, u16};
use futures::future::err;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use random_string::generate;
use reqwest::{ClientBuilder, Proxy, StatusCode};
use reqwest::Client;
use reqwest::cookie::Jar;
use reqwest::header::HeaderMap;
use rocket::figment::Source::Code;
use rocket::request;
use rocket::yansi::Paint;
use serde_json::{from_str, json, Map, Number, Value};
use tokio::sync::RwLock;
use url::Url;
use warp::http::HeaderValue;
use self::audio_challenge::AudioChallenge;
use self::bda::structs::ArkoseFingerprint;
use self::challenge::Challenge;
use self::encryption::encrypt;
use self::headers::generate_headers;
use self::structs::{EncryptionKeyResponse, FunCaptchaRequest, SolvedCaptchaResponse};
use self::structs::game_struct::{AnswerResponse, Game};
use self::structs::session_structs::SessionResponse;
use crate::{ARGUMENTS, DORTCAP_CONFIG, PROXIES, SOLVED};
use crate::captcha::arkose_funcaptcha::bda::firefox::get_encrypted_firefox_bda;
use crate::captcha::arkose_funcaptcha::headers::generate_headers_capi;
use crate::commons::error::DortCapError::{CodeErr, DetailedInternalErr};
use crate::commons::error::DortCapResult;
use crate::commons::REDIS_RUNTIME;

pub mod structs;
pub mod bda;
pub mod encryption;
mod breakers;
mod challenge;
mod headers;
mod audio_challenge;
mod imageprocessing;


#[derive(Debug)]
pub struct ArkoseSession {
    bda: ArkoseFingerprint,
    request: FunCaptchaRequest,
    headers: HeaderMap,
    capi_version: String,
    client: Client
}

async fn breakers_gt3(location: (i32, i32), large: bool) -> Value {
    let x = if large { 450 } else { 300 };
    let y = if large { 150 } else { 200 };
    let px = format!("{:.2}", location.0 / x);
    let py = format!("{:.2}", location.1 / y);
    let mut result: Map<String, Value> = Map::new();
    result.insert(String::from("px"), Value::String(px));
    result.insert(String::from("py"), Value::String(py));
    result.insert(String::from("x"), Value::Number(Number::from(location.0)));
    result.insert(String::from("y"), Value::Number(Number::from(location.1)));
    Value::Object(result)
}

async fn get_pos(tile_number: i32, large: bool) -> (i32, i32) {
    let grid_size = 3;
    let tile_size = if large { 150 } else { 100 };
    let row = tile_number / grid_size;
    let col = tile_number % grid_size;
    let pos_x = i32(0..tile_size);
    let pos_y = i32(0..tile_size);
    let x = col * tile_size + pos_x;
    let y = row * tile_size + pos_y;
    (x, y)
}

fn round(x: f64, decimals: u32) -> f64 {
    let y = 10i64.pow(decimals) as f64;
    (x * y).round() / y
}
const ENCODING_SET: &AsciiSet = &CONTROLS.add(b'%').add(b'$').add(b'&').add(b'+').add(b',')
    .add(b'/').add(b':').add(b';').add(b'=').add(b'?').add(b'@').add(b'<')
    .add(b'>').add(b'#').add(b'%').add(b' ');

lazy_static!{
    static ref CAPI_VERSIONS_CACHE: RwLock<HashMap<String, String>> = RwLock::default();
}

impl ArkoseSession {

    // new arkose session
    pub async fn new(mut request: FunCaptchaRequest) -> DortCapResult<Self> {
        let mut client: Client;
        let proxies = PROXIES.read().await;
        let proxy = request.proxy.as_deref().unwrap_or(fastrand::choice(&*proxies).ok_or(CodeErr(0x01, "PROXIES"))?);
        let mut client_builder = ClientBuilder::new()
            .timeout(Duration::from_secs(20));
        let proxy = request.proxy.as_deref().unwrap_or(proxy);
        client_builder = client_builder.proxy(Proxy::all(proxy.replace("%SESSION_ID%", &*generate(23, "abcdef1234567890")).replace("%RND_PORT%", &*u16::to_string(&u16(10000..20000))))?);
        client = client_builder.build()?;
        drop(proxies);
        if !CAPI_VERSIONS_CACHE.read().await.contains_key(&request.site_key) {
            let href_raw = client.get(format!("https://funcaptcha.com/v2/{}/api.js", &request.site_key)).headers(generate_headers_capi().await?).send().await?.text().await?;
            let mut href_raw = href_raw.split("file:\"");
            let _ = href_raw.next().ok_or(DetailedInternalErr("CAPI_VERSION_FETCH_FAILED"))?;
            let href_pt1 = href_raw.next().ok_or(DetailedInternalErr("CAPI_VERSION_FETCH_FAILED"))?;
            let _ = href_pt1.split("\"").next().ok_or(DetailedInternalErr("CAPI_VERSION_FETCH_FAILED"))?;
            let capi_version = href_pt1.split("\"").next().ok_or(DetailedInternalErr("CAPI_VERSION_FETCH_FAILED"))?;
            CAPI_VERSIONS_CACHE.write().await.insert(request.site_key.clone(), String::from(capi_version));
        }
        let capi_version = CAPI_VERSIONS_CACHE.read().await.get(&request.site_key).ok_or(DetailedInternalErr("CAPI_VERSION_FETCH_FAILED"))?.clone();
        let bda = get_encrypted_firefox_bda(&*format!("https://{}/v2/{}", request.arkose_api_url, capi_version), &mut request.bda_template).await?;
        let headers = generate_headers(&*capi_version, &*request.arkose_api_url, &bda.headers).await?;
        Ok(ArkoseSession {
            capi_version,
            request,
            headers,
            bda,
            client
        })
    }

    // retrieve arkose session token
    async fn get_session_token(&self) -> DortCapResult<SessionResponse>  {
        let r = self.client.get(format!("https://iframe.arkoselabs.com/{0}", self.request.site_key)).send().await?;
        let rnd = f64::to_string(&round(fastrand::f64(), 16));
        let mut data = IndexMap::new();
        // fingerprint
        data.insert("bda", &*self.bda.fingerprint_enc);
        // site key
        data.insert("public_key", &*self.request.site_key);
        // site url
        data.insert("site", &*self.request.site_url);
        // user agent
        data.insert("userbrowser", &*self.bda.user_agent);
        // site api version, differentiates for lower security sites.
        data.insert("capi_version", &*self.capi_version.split("/").next().ok_or(DetailedInternalErr("CAPI_FETCH_FAILED"))?);
        // always inline unless user has screen reader.
        data.insert("capi_mode", "inline");
        // default theme always, no check for it so why bother lol
        data.insert("style_theme", "default");
        // arkose rnd value, used on aggression to detect repeated requests
        data.insert("rnd", &*rnd);
        // data.insert("language", "en");
        // arkose customer blob, encrypted by the site, sent to arkose by the client, and, yes there's a better way to urldecode it i just don't give a fuck :/
        let blob = Option::as_deref(&self.request.data).unwrap_or("undefined").replace(" ", "%2B");
        data.insert("data[blob]", &*blob);
        // data.insert("data[id]", "customer_transparent");
        let mut headers_cloned = self.headers.clone();
        let mut time = SystemTime::UNIX_EPOCH.elapsed()?.as_secs();
        time -= time % 21600;
        headers_cloned.insert("x-ark-esync-value", time.to_string().parse()?);
        let mut new_data = String::new();
        for x in &data {
            new_data += &*format!("{0}={1}&", x.0, utf8_percent_encode(x.1, ENCODING_SET).collect::<String>());
        }
        #[allow(unused)]
        new_data.split_off(new_data.len() - 1).truncate(0);
        let response = self.client.post(format!("{}/fc/gt2/public_key/{}", self.request.arkose_api_url, self.request.site_key))
            .body(new_data)
            .headers(headers_cloned)
            .send().await?
            .text().await?;
        // println!("{}", response);
        Ok(from_str(&*response)?)
    }

    // retrieve funcaptcha challenge
    async fn get_challenge(&self, spoof: bool, session_token_full: &str, session_token: &str, region: &str) -> DortCapResult<Game> {
        let arkose_url = &*self.request.arkose_api_url;
        let mut new_headers = self.headers.clone();
        // new_headers.insert("X-Requested-Id", encrypt("{\"return_security_info\":1,\"nojs_fb_type\":11}", &*format!("REQUESTED{session_token}ID"))?.to_string().parse()?);
        new_headers.insert("Referer", HeaderValue::try_from(format!("{arkose_url}/fc/assets/ec-game-core/game-core/1.19.1/standard/index.html?session={}", session_token_full.replace("|", "&"))).unwrap());
        let mut data = IndexMap::new();
        data.insert("token", session_token);
        data.insert("sid", region);
        data.insert("render_type", if spoof { "liteJS" } else { "canvas" });
        data.insert("isAudioGame", if ARGUMENTS.try_audio || self.request.audio {
            "true"
        } else {
            "false"
        });
        data.insert("analytics_tier", "40");
        data.insert("is_compatibility_mode", "false");
        data.insert("apiBreakerVersion", "green");
        let game_token_response = self.client.post(format!("{}/fc/gfct/", self.request.arkose_api_url))
            .form(&data)
            .headers(new_headers)
            .send().await?
            .text().await?;
        Ok(from_str(&game_token_response)?)
    }

    // submit challenge answers
    async fn submit_answer(&self, dapi_script: &str, answers: &Vec<Value>, game_type: i32, session_token: &str, game_token: &str, region: &str) -> DortCapResult<AnswerResponse> {
        let mut headers = self.headers.clone();
        headers.insert("Referer", HeaderValue::try_from("https://client-api.arkoselabs.com/fc/assets/ec-game-core/game-core/1.20.0/standard/index.html")?);
        headers.insert("X-Requested-With", HeaderValue::try_from("XMLHttpRequest")?);
        let mut data = BTreeMap::new();
        let dat2 = json!(answers).to_string();
        let enc2 = encrypt(&dat2, session_token)?.to_string();
        data.insert("session_token", session_token);
        data.insert("game_token", game_token);
        let dat1 = &*json!(breakers::get_answers(self.headers.clone(), &self.client, dapi_script, game_type, answers, &session_token.to_string()).await?).to_string();
        let enc1 = &*encrypt(dat1, session_token)?.to_string();
        if game_type == 4 {
            data.insert("tguess", enc1);
        }
        data.insert("guess", &*enc2);
        data.insert("sid", region);
        data.insert("render_type", "canvas");
        data.insert("analytics_tier", "40");
        data.insert("bio", "eyJtYmlvIjoiIiwidGJpbyI6IiIsImtiaW8iOiIifQ==");
        let response = self.client.post(format!("{}/fc/ca/", self.request.arkose_api_url)).form(&data).headers(headers).send().await?.text().await?;
        Ok(from_str(&response)?)
    }

    // retrieve encryption / decryption key from arkose
    async fn fetch_encryption_key(&self, session_token: &str, game_token: &str, region: &str) -> DortCapResult<EncryptionKeyResponse> {
        let mut data = HashMap::new();
        data.insert("session_token", session_token);
        data.insert("game_token", game_token);
        data.insert("sid", region);
        let ekey_token_response = self.client.post(format!("{}/fc/ekey/", self.request.arkose_api_url)).form(&data).headers(self.headers.clone()).send().await?.text().await?;
        Ok(from_str(&ekey_token_response)?)
    }

    async fn parse_region_from_token(&self, token: &str) -> DortCapResult<String> {
        let strs: Vec<&str> = token.split("|r=").collect();
        Ok(strs[1].split("|").next().ok_or(DetailedInternalErr("TOKEN_PARSE"))?.to_string())
    }

    async fn split_token(&self, token: &str) -> DortCapResult<String> {
        let strs: Vec<&str> = token.split("|").collect();
        Ok(strs[0].to_string())
    }


    pub async fn solve(&self) -> DortCapResult<SolvedCaptchaResponse> {
        let session = self.get_session_token().await?;
        if session.error.is_some() {
            return Err(DetailedInternalErr("SESSION_TOKEN"));
        }
        let token = &*session.token.ok_or(DetailedInternalErr("TOKEN_NULL"))?;
        if token.contains("sup=1|") {
            *SOLVED.write().await += 1;
            return Ok(SolvedCaptchaResponse {
                variant: Some(String::from("suppressed")),
                token: Some(String::from(token)),
                solved: Some(true),
                waves: Some(0),
                notes: None,
                ip_used: None,
                error: None,
            });
        }
        let region = self.parse_region_from_token(token).await?;
        let split_token = self.split_token(token).await?;
        let game: Game = self.get_challenge(false, token, &split_token, &region).await?;
        let game_data = game.game_data.ok_or(CodeErr(0x01, "CHALLENGE"))?;
        let game_token = game.challenge_id.ok_or(CodeErr(0x02, "CHALLENGE"))?;
        let game_type = game_data.game_type.ok_or(CodeErr(0x03, "CHALLENGE"))?;
        let custom_gui = game_data.custom_gui.ok_or(CodeErr(0x04, "CHALLENGE"))?;
        // for some reason any other way I tried this didn't work, unwrapping failed on a non-None value :skull:
        let mut instruction_string = game_data.instruction_string;
        if instruction_string.is_none() {
            instruction_string = custom_gui.instruction_string;
        }
        if instruction_string.is_none() {
            instruction_string = game_data.game_variant;
        }
        let instruction_string = instruction_string.ok_or(CodeErr(0x01, "INSTRUCTION"))?;
        let mut answers = Vec::new();
        let decryption_key_opt = self.fetch_encryption_key(&split_token, &game_token, &region).await;
        let mut decryption_key;
        // aids but it works.
        if decryption_key_opt.is_err() {
            decryption_key = EncryptionKeyResponse{ error: Some(String::from("No key present, Continue.")), decryption_key: None }
        } else {
            decryption_key = decryption_key_opt?;
        }
        let mut answer_response = AnswerResponse {
            response: None,
            solved: Some(false),
            decryption_key: None,
            incorrect_guess: None,
        };
        let waves: i32;
        if game_type != 101 {
            let images = custom_gui._challenge_imgs.ok_or(CodeErr(0x01, "IMAGES"))?;
            waves = images.len() as i32;
            println!("t: {} w: {} g: {}", split_token, waves, instruction_string.to_string());
            let mut challenges: Vec<Challenge> = Vec::new();
            for image_url in &images {
                let challenge = Challenge::new(&self.client, &self.headers, game_data.game_difficulty.unwrap_or(6) as u8, game_type as u8, &instruction_string, &image_url, &decryption_key.decryption_key).await?;
                if game_type == 4 {
                    answers.push(json!({"index": challenge.selected_tile}));
                } else {
                    answers.push(json!(breakers_gt3(get_pos(challenge.selected_tile as i32, challenge.grid_size == 450).await, challenge.grid_size == 450).await));
                }
                challenges.push(challenge);
                answer_response = self.submit_answer(game.dapi_breakers.as_deref().unwrap_or("NOT_REQUIRED"), &mut answers, game_type, &split_token, &game_token, &region).await?;
                if decryption_key.error.is_none() {
                    decryption_key = EncryptionKeyResponse {
                        error: None,
                        decryption_key: answer_response.decryption_key
                    }
                }
            }
            if answer_response.solved.unwrap_or(false) {
                *SOLVED.write().await += 1;
                for challenge in challenges {
                    REDIS_RUNTIME.spawn(async move {
                        let _result = challenge.save_tiles().await;
                    });
                }
            }
        } else {
            let clips = game.audio_challenge_urls.ok_or(CodeErr(0x01, "SOUND"))?;
            waves = clips.len() as i32;
            let mut challenges: Vec<AudioChallenge> = Vec::new();
            for clip in &clips {
                let challenge = AudioChallenge::new(&self.headers, &self.client, clip, &instruction_string, decryption_key.decryption_key.as_deref()).await?;
                answers.push(json!(challenge.selected_clip));
                challenges.push(challenge);
                answer_response = self.submit_answer(game.dapi_breakers.as_deref().unwrap_or("NOT_REQUIRED"), &answers, game_type, &split_token, &game_token, &region).await?;
                // println!("{:?}", answer_response);
                if decryption_key.error.is_none() {
                    decryption_key = EncryptionKeyResponse {
                        error: None,
                        decryption_key: answer_response.decryption_key
                    }
                }
            }
            if answer_response.solved.ok_or(CodeErr(0x01, "SOLVE"))? {
                *SOLVED.write().await += 1;
                for mut challenge in challenges {
                    REDIS_RUNTIME.spawn(async move {
                        challenge.save_audio().await;
                    });
                }
            }
        }
        Ok(SolvedCaptchaResponse {
            variant: Some(instruction_string),
            token: Some(String::from(token)),
            solved: answer_response.solved,
            waves: Some(waves),
            notes: None,
            ip_used: None,
            error: None,
        })
    }
}


