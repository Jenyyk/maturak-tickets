use image::codecs::png::PngEncoder;
use image::{ImageBuffer, Luma};
use qrcode::{types::EcLevel, QrCode};
use std::io::Cursor;

pub fn generate_qr_code(qr_content: &str) -> Vec<u8> {
    print!("Generating hash: ");
    println!("{}", qr_content);

    print!("Generating QR code... ");
    let code = QrCode::with_error_correction_level(qr_content, EcLevel::H).unwrap();

    let image = code.render::<Luma<u8>>().build();
    let mut buf = Vec::new();

    let encoder = PngEncoder::new(&mut buf);
    image
        .write_with_encoder(encoder)
        .expect("Failed to write image to buffer");

    print!("done with ");

    style_qr_code(buf)
}

use image::{DynamicImage, GenericImage, ImageReader, RgbaImage};

fn style_qr_code(qr_code_buf: Vec<u8>) -> Vec<u8> {
    let qr_image = ImageReader::new(Cursor::new(qr_code_buf))
        .with_guessed_format()
        .expect("Failed to read QR buffer")
        .decode()
        .expect("Failed to decode QR image")
        .into_rgba8();

    let header_image = ImageReader::open("qr_header.png")
        .expect("Failed to load header image")
        .decode()
        .expect("Failed to decode header image")
        .into_rgba8();

    let (_, header_height) = header_image.dimensions();
    let (qr_width, qr_height) = qr_image.dimensions();

    let total_height = header_height + qr_height;
    let mut final_image: RgbaImage = ImageBuffer::new(qr_width, total_height);

    final_image
        .copy_from(&header_image, 0, 0)
        .expect("Failed to copy header");
    final_image
        .copy_from(&qr_image, 0, header_height)
        .expect("Failed to copy QR code");

    let mut buf = Vec::new();
    let encoder = PngEncoder::new(&mut buf);
    DynamicImage::ImageRgba8(final_image)
        .write_with_encoder(encoder)
        .expect("Failed to encode final image");

    buf
}
