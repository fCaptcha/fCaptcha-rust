use hex::ToHex;
use image::DynamicImage;
use image_hasher::{
    HashAlg::Median, HasherConfig
};

pub async fn hash_image(image: &DynamicImage) -> String {
    let hasher = HasherConfig::new()
        .hash_size(128, 128)
        .hash_alg(Median)
        .preproc_dct()
        .to_hasher();
    hasher.hash_image(image).into_inner().encode_hex_upper()
}