use std::str::FromStr;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use hex::ToHex;
use lazy_static::lazy_static;
use md5::compute;
use redis::{AsyncCommands, RedisResult};
use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use crate::commons::error::DortCapResult;
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
    let mut decoder = Decoder::new(data);
    let mut num = 0f32;
    let mut i = 0;
    while let Some(frame) = decoder.next() {
        if let Audio(audio) = frame {
            if i % 6 == 0 {
                for sample in audio.samples() {
                    num += sample;
                }
            }
            i += 1;
        }
    }
    num.to_string()
}


async fn download_clips(client: &Client, url: &str, decryption_key: Option<&str>) -> DortCapResult<Vec<u8>> {
    let mut headers = HeaderMap::new();
    headers.insert("Accept", HeaderValue::try_from("*/*")?);
    headers.insert("Accept-Encoding", HeaderValue::try_from("gzip, deflate, br")?);
    headers.insert("Content-Type", HeaderValue::try_from("application/x-www-form-urlencoded; charset=UTF-8")?);
    headers.insert("Origin", HeaderValue::try_from("https://client-api.arkoselabs.com")?);
    headers.insert("Referer", HeaderValue::try_from("https://client-api.arkoselabs.com/")?);
    headers.insert("Sec-Fetch-Dest", HeaderValue::try_from("empty")?);
    headers.insert("Sec-Fetch-Mode", HeaderValue::try_from("cors")?);
    headers.insert("Sec-Fetch-Site", HeaderValue::try_from("same-origin")?);
    headers.insert("User-Agent", HeaderValue::try_from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36")?);
    let bytes = client.get(url).headers(headers).send().await?.bytes().await?;
    if decryption_key.is_some() {
        return Ok(BASE64_STANDARD.decode(cryptojs_decrypt(&*String::from_utf8(bytes.to_vec())?, decryption_key.unwrap())?)?);
    }
    Ok(Vec::from(bytes))
}

impl AudioChallenge {
    pub async fn new(client: &Client, audio_link: &str, variant: &str, enc_key: Option<&str>) -> DortCapResult<AudioChallenge> {
        let clips = download_clips(client, audio_link, enc_key).await?.split_off(1);
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