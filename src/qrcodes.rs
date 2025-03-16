use std::hash::{DefaultHasher, Hasher, Hash};
pub fn generate_hash(t: &str) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

use qrcode::{QrCode, types::EcLevel};
use image::{Luma, ImageBuffer};
use image::codecs::png::PngEncoder;
use std::io::Cursor;

pub fn generate_qr_code(receiver_mail: &str) -> Vec<u8> {
    print!("Generating hash: ");
    let qr_content = generate_hash(receiver_mail).to_string();
    println!("{}", qr_content);

    print!("Generating QR code... ");
    let code = QrCode::with_error_correction_level(qr_content, EcLevel::H).unwrap();

    let image = code.render::<Luma<u8>>().build();
    let mut buf = Vec::new();

    let encoder = PngEncoder::new(&mut buf);
    image.write_with_encoder(encoder).expect("Failed to write image to buffer");

    println!("done");

    buf
}
