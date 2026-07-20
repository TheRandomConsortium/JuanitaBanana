use base64::Engine;

pub fn get_outfit_font_b64() -> String {
    let bytes = include_bytes!("../../../templates/fonts/Outfit.ttf");
    base64::engine::general_purpose::STANDARD.encode(bytes)
}
