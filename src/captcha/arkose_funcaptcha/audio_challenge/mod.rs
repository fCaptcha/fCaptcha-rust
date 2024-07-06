use std::str::FromStr;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use hex::ToHex;
use lazy_static::lazy_static;
use md5::compute;
use redis::{AsyncCommands, RedisResult};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use crate::commons::error::FCaptchaResult;
use super::encryption::cryptojs_decrypt;
use crate::IMAGE_DATABASE;

#[derive(Clone)]
pub struct AudioChallenge {
    pub data: Vec<u8>,
    pub variant: String,
    pub selected_clip: i32,
    pub hash: String
}

lazy_static! {
    static ref CLIENT: Client = reqwest::ClientBuilder::new().build().unwrap();
}

use rmp3::{
    Frame::Audio,
    Decoder
};

pub fn hash_audio(data: &Vec<u8>) -> String {
    md5::compute(data).0.encode_hex()
}


async fn download_clips(header_map: &HeaderMap, client: &Client, url: &str, decryption_key: Option<&str>) -> FCaptchaResult<Vec<u8>> {
    let bytes = client.get(url).headers(header_map.clone()).send().await?.bytes().await?;
    if decryption_key.is_some() {
        return Ok(BASE64_STANDARD.decode(cryptojs_decrypt(&*String::from_utf8(bytes.to_vec())?, decryption_key.unwrap())?)?);
    }
    Ok(Vec::from(bytes))
}

impl AudioChallenge {
    pub async fn new(header_map: &HeaderMap, client: &Client, audio_link: &str, variant: &str, enc_key: Option<&str>) -> FCaptchaResult<AudioChallenge> {
        let clips = download_clips(header_map, client, audio_link, enc_key).await?.split_off(1);
        let hash = hash_audio(&clips);
        let key = format!("Game Type 4:Audio Hashes (3 Options):{}:{}", variant, hash);
        let mut answer = fastrand::i32(1..4);
        match IMAGE_DATABASE.get().await.to_owned().get::<String, String>(key).await {
            Ok(audio_answer) => {
                println!("good 4-a {variant} {audio_answer}");
                answer = i32::from_str(&audio_answer)?;
            }
            Err(_) => (),
        }
        Ok(Self {
            data: clips,
            variant: String::from(variant),
            selected_clip: answer,
            hash,
        })
    }
    pub async fn save_audio(&mut self) {
        let key = format!("Game Type 4:Audio Hashes (3 Clips):{}:{}", &self.variant, &self.hash);
        loop {
            let clip_save_result: RedisResult<String> = IMAGE_DATABASE.get().await.to_owned().set(key.to_owned(), self.selected_clip).await;
            if clip_save_result.is_ok() {
                break;
            }
        }
    }
}