use base64::Engine;
use rand::Rng;

pub fn get_monero_qr_b64() -> String {
    let bytes = include_bytes!("../../assets/monerowallet.png");
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub fn get_icon_b64() -> String {
    let bytes = include_bytes!("../../assets/icon.png");
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub fn generate_random_noise_bmp_b64() -> String {
    let mut rng = rand::thread_rng();
    let width = 256;
    let height = 256;
    let pixel_data_size = width * height * 4;
    let mut bmp = Vec::with_capacity(54 + pixel_data_size);

    bmp.extend_from_slice(b"BM");
    let file_size = (54 + pixel_data_size) as u32;
    bmp.extend_from_slice(&file_size.to_le_bytes());
    bmp.extend_from_slice(&[0, 0, 0, 0]);
    bmp.extend_from_slice(&(54u32).to_le_bytes());

    bmp.extend_from_slice(&(40u32).to_le_bytes());
    bmp.extend_from_slice(&(width as u32).to_le_bytes());
    bmp.extend_from_slice(&(height as i32).to_le_bytes());
    bmp.extend_from_slice(&(1u16).to_le_bytes());
    bmp.extend_from_slice(&(32u16).to_le_bytes());
    bmp.extend_from_slice(&(0u32).to_le_bytes());
    bmp.extend_from_slice(&(pixel_data_size as u32).to_le_bytes());
    bmp.extend_from_slice(&(2835u32).to_le_bytes());
    bmp.extend_from_slice(&(2835u32).to_le_bytes());
    bmp.extend_from_slice(&(0u32).to_le_bytes());
    bmp.extend_from_slice(&(0u32).to_le_bytes());

    let mut raw_pixels = vec![0u8; pixel_data_size];
    rng.fill(&mut raw_pixels[..]);
    bmp.extend_from_slice(&raw_pixels);

    base64::engine::general_purpose::STANDARD.encode(&bmp)
}
