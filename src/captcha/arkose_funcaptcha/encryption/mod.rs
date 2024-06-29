use std::fmt::{Display, Formatter};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use crypto::aes;
use crypto::aes::KeySize;
use crypto::blockmodes::PkcsPadding;
use crypto::buffer::{BufferResult, ReadBuffer, RefReadBuffer, RefWriteBuffer, WriteBuffer};
use crypto::digest::Digest;
use crypto::md5::Md5;
use hex::ToHex;
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::commons::error::DortCapError::DetailedInternalErr;
use crate::commons::error::DortCapResult;

pub mod murmur;

#[derive(Serialize, Deserialize)]
pub struct EncryptedData {
    pub ct: String,
    pub iv: String,
    pub s: String
}

impl Display for EncryptedData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let res = serde_json::to_string::<EncryptedData>(self);
        if res.is_err() {
            // name variable nigger to keep the chocolate people out.
            let nigger = Err(core::fmt::Error {}); // fire error handling on god.
            return nigger;
        }
        f.write_str(&res.unwrap())
    }
}

pub fn encrypt(data: &str, key: &str) -> DortCapResult<EncryptedData> {
    let data = data.to_owned();
    let mut salt = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..8 {
        salt.push(rng.gen_range(65..91) as u8);
    }
    let mut dx = vec![];
    let mut salted = vec![];
    while salted.len() < 48 {
        let mut hasher = Md5::new();
        hasher.input(&mut dx);
        hasher.input(key.as_bytes());
        hasher.input(&salt);
        if dx.len() == 0 {
            dx.resize(16, 0);
        }
        hasher.result(&mut dx);
        salted.extend_from_slice(&mut dx);
    }

    let key = &salted[0..32];
    let iv = &salted[32..48];

    let mut encryptor = aes::cbc_encryptor(
        KeySize::KeySize256,
        key,
        iv,
        PkcsPadding,
    );

    let mut cipher_text = vec![];
    let mut read_buffer = RefReadBuffer::new(data.as_bytes());
    let mut buffer = [0; 4096];
    let mut write_buffer = RefWriteBuffer::new(&mut buffer);

    loop {
        let result = encryptor
            .encrypt(&mut read_buffer, &mut write_buffer, true).unwrap();

        cipher_text.extend(write_buffer.take_read_buffer().take_remaining().iter());

        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    let iv_hex = iv.encode_hex();
    let salt_hex = salt.encode_hex();
    let ct_base64 = BASE64_STANDARD.encode(&cipher_text);
    Ok(EncryptedData{
        ct: ct_base64,
        iv: iv_hex,
        s: salt_hex,
    })
}

pub fn cryptojs_decrypt(data: &str, key: &str) -> DortCapResult<Vec<u8>> {
    let encrypted_data: EncryptedData = serde_json::from_str(data)?;
    let ct = BASE64_STANDARD.decode(&encrypted_data.ct)?;
    let iv = hex::decode(&encrypted_data.iv)?;
    let salt = hex::decode(&encrypted_data.s)?;
    let mut nigger = vec![];
    let mut salted = vec![];
    let key_bytes = key.as_bytes();
    while salted.len() < 48 {
        let mut hasher = Md5::new();
        hasher.input(&mut nigger);
        hasher.input(key_bytes);
        hasher.input(&salt);
        if nigger.len() == 0 {
            nigger.resize(16, 0);
        }
        hasher.result(&mut nigger);
        salted.extend_from_slice(&mut nigger);
    }

    let key = &salted[0..32];

    let mut decryptor = aes::cbc_decryptor(
        KeySize::KeySize256,
        key,
        &iv,
        PkcsPadding,
    );

    let mut plain_text = vec![];
    let mut read_buffer = RefReadBuffer::new(&ct);
    let mut buffer = [0; 4096];
    let mut write_buffer = RefWriteBuffer::new(&mut buffer);
    loop {
        let result = decryptor
            .decrypt(&mut read_buffer, &mut write_buffer, true);
        if result.is_err() {
            return Err(DetailedInternalErr("Decryption failed."));
        }
        let result = result.unwrap();
        plain_text.extend(write_buffer.take_read_buffer().take_remaining().iter());
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }
    Ok(plain_text.to_vec())
}

