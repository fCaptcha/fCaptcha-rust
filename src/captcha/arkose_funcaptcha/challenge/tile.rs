use image::DynamicImage;
use image_hasher::{HashAlg, HasherConfig, ImageHash};
use image_hasher::HashAlg::DoubleGradient;

use crate::commons::error::DortCapResult;
use crate::DORTCAP_CONFIG;

pub async fn hash_image(img: &DynamicImage) -> DortCapResult<(ImageHash, String)> {
    let size = DORTCAP_CONFIG.hashing.hash_size;
    let hasher_config = HasherConfig::new().hash_size(size, size).hash_alg(HashAlg::Median).preproc_dct();
    let h = hasher_config.to_hasher().hash_image(img);
    Ok((h.clone(), h.to_base64()))
}

#[derive(Clone)]
pub struct Tile {
    pub hash: String,
    pub hash_raw: ImageHash,
    pub width: u32,
    pub height: u32
}

impl Tile {
    pub async fn new(contents: &DynamicImage, instr: bool) -> DortCapResult<Self>  {
        let raw = hash_image(contents).await?;
        let h = raw.0;
        let t = raw.1;
        Ok(Tile {
            hash: t,
            width: contents.width(),
            height: contents.height(),
            hash_raw: h
        })
    }
}