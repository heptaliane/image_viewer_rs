use std::fs;
use std::path::Path;

use base64;
use mime;

pub fn try_get_source_image(filename: &String) -> Result<String, String> {
    match Path::new(filename).extension() {
        Some(ext) => {
            let mimetype: mime::Mime = match ext.to_ascii_lowercase().to_str().unwrap() {
                "bmp" => mime::IMAGE_BMP,
                "jpg" => mime::IMAGE_JPEG,
                "jpeg" => mime::IMAGE_JPEG,
                "png" => mime::IMAGE_PNG,
                "gif" => mime::IMAGE_GIF,
                _ => {
                    return Err(format!("Unsupported file: {}", ext.to_str().unwrap()));
                }
            };

            match fs::read(filename) {
                Ok(data) => {
                    let b64data = base64::encode(data);
                    Ok(format!("data:{};base64,{}", mimetype.to_string(), b64data))
                }
                Err(err) => Err(format!("{:?}", err)),
            }
        }
        None => Err("No extension".to_string()),
    }
}
