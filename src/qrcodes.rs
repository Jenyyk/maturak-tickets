use image::codecs::png::PngEncoder;
use image::{DynamicImage, GenericImage, ImageReader, RgbaImage};
use qrcode::{types::EcLevel, QrCode};
use std::io::Cursor;

pub fn generate_qr_code(qr_content: &str, style_design: &str) -> Vec<u8> {
    println!("Generating hash: {}", qr_content);

    // Generate the QR code
    let code = QrCode::with_error_correction_level(qr_content, EcLevel::H).unwrap();
    let image = code
        .render::<image::Luma<u8>>()
        .min_dimensions(500, 500)
        .build();
    let mut buf = Vec::new();

    // Encode QR code into a PNG buffer
    let encoder = PngEncoder::new(&mut buf);
    image
        .write_with_encoder(encoder)
        .expect("Failed to write image to buffer");

    // Style the QR code by merging with header image
    style_qr_code(buf, style_design)
}

fn style_qr_code(qr_code_buf: Vec<u8>, header_path: &str) -> Vec<u8> {
    let qr_image = ImageReader::new(Cursor::new(qr_code_buf))
        .with_guessed_format()
        .expect("Failed to read QR buffer")
        .decode()
        .expect("Failed to decode QR image")
        .into_rgba8();

    let header_image = ImageReader::open(format!("qr_header_{}.png", header_path))
        .expect("Failed to load header image")
        .decode()
        .expect("Failed to decode header image")
        .into_rgba8();

    let (header_width, header_height) = header_image.dimensions();
    let (qr_width, qr_height) = qr_image.dimensions();

    let total_width = header_width + qr_width;
    let total_height = std::cmp::max(header_height, qr_height);

    print!("dimensions: {}x{} ", total_width, total_height);

    let mut final_image: RgbaImage = RgbaImage::new(total_width, total_height);

    final_image
        .copy_from(&header_image, 0, 0)
        .expect("Failed to copy header");

    final_image
        .copy_from(&qr_image, header_width, 0)
        .expect("Failed to copy QR code");

    let photon_image = open_image_from_rgba8(&final_image);

    // APPLY photon_rs EFFECTS HERE

    let final_image_rgba8 = convert_photon_to_rgba8(photon_image);

    // Encode final image into a PNG buffer
    let mut buf = Vec::new();
    let encoder = PngEncoder::new(&mut buf);
    DynamicImage::ImageRgba8(final_image_rgba8)
        .write_with_encoder(encoder)
        .expect("Failed to encode final image");

    buf
}

fn open_image_from_rgba8(image: &RgbaImage) -> photon_rs::PhotonImage {
    let (width, height) = image.dimensions();
    let raw_pixels = image.to_vec();

    photon_rs::PhotonImage::new(raw_pixels, width, height)
}

fn convert_photon_to_rgba8(photon_image: photon_rs::PhotonImage) -> RgbaImage {
    let width = photon_image.get_width();
    let height = photon_image.get_height();
    let raw_pixels = photon_image.get_raw_pixels();

    RgbaImage::from_raw(width, height, raw_pixels).expect("Failed to convert back to RGBA8")
}
