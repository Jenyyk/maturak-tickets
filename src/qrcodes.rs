use image::codecs::png::PngEncoder;
use image::{DynamicImage, GenericImage, ImageReader, RgbaImage};
use photon_rs::PhotonImage;
use qrcode::{types::EcLevel, QrCode};
use std::io::Cursor;

use crate::database::Database;

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
    style_qr_code(buf, style_design, qr_content)
}

fn style_qr_code(qr_code_buf: Vec<u8>, header_path: &str, hash: &str) -> Vec<u8> {
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
    let (qr_width, _qr_height) = qr_image.dimensions();

    let mut full_qr_image: RgbaImage = RgbaImage::new(header_width, header_height);

    full_qr_image
        .copy_from(&qr_image, header_width - qr_width, 0)
        .expect("Failed to copy QR code");

    let mut photon_qr_image = open_image_from_rgba8(&full_qr_image);
    let photon_header_image = open_image_from_rgba8(&header_image);
    photon_rs::multiple::blend(&mut photon_qr_image, &photon_header_image, "normal");

    let ticket_num = Database::get_ticket_count() + 1;
    let mut ticket_text = String::new();
    ticket_text.push('D');
    ticket_text.push_str(match ticket_num {
        ..=9 => "00",
        10..=99 => "0",
        _ => "",
    });
    ticket_text.push_str(&ticket_num.to_string());

    // APPLY photon_rs EFFECTS HERE
    TextBuilder::new(&mut photon_qr_image)
        .text(ticket_text)
        .left(20)
        .bottom(20)
        .size(80.0)
        .color((24, 24, 24))
        .write();
    TextBuilder::new(&mut photon_qr_image)
        .text(hash)
        .right(100)
        .bottom(10)
        .size(30.0)
        .color((0, 0, 0))
        .write();

    let final_image_rgba8 = convert_photon_to_rgba8(photon_qr_image);

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

use photon_rs::Rgb;
struct TextBuilder<'a> {
    image: &'a mut PhotonImage,
    text: Option<String>,
    top: Option<i32>,
    left: Option<i32>,
    bottom: Option<i32>,
    right: Option<i32>,
    color: Option<Rgb>,
    size: Option<f32>
}
#[allow(dead_code)]
impl<'a> TextBuilder<'a> {
    fn new(image: &'a mut PhotonImage) -> Self {
        TextBuilder {
            image,
            text: None,
            top: None,
            left: None,
            bottom: None,
            right: None,
            color: None,
            size: None
        }
    }

    fn text(mut self, val: impl Into<String>) -> Self {
        let val = val.into();
        self.text = Some(val);
        self
    }

    fn top(mut self, val: i32) -> Self {
        self.bottom = None;
        self.top = Some(val);
        self
    }
    fn left(mut self, val: i32) -> Self {
        self.right = None;
        self.left = Some(val);
        self
    }
    fn bottom(mut self, val: i32) -> Self {
        self.top = None;
        self.bottom = Some(val);
        self
    }
    fn right(mut self, val: i32) -> Self {
        self.left = None;
        self.right = Some(val);
        self
    }

    fn color(mut self, color: (u8, u8, u8)) -> Self {
        self.color = Some(Rgb::new(color.0, color.1, color.2));
        self
    }

    fn size(mut self, val: f32) -> Self {
        self.size = Some(val);
        self
    }

    fn estimate_width(&self) -> i32 {
        let text_len = self.text.as_ref().unwrap_or(&String::from("")).len();
        let size = self.size.unwrap_or(0.0);
        (text_len as f32 * size * (1.0/2.0)) as i32
    }

    fn write(self) {
        let image_width = self.image.get_width();
        let image_height = self.image.get_height();

        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let size = self.size.unwrap_or(50.0);

        if let Some(top) = self.top { y = top; }
        if let Some(bot) = self.bottom { y = (image_height as i32) - (size as i32) - bot; }

        if let Some(left) = self.left { x = left; }
        if let Some(right) = self.right { x = (image_width as i32) - self.estimate_width() - right; }

        photon_rs::text::draw_text(self.image, &self.text.unwrap_or(String::from("no text")), x, y, size, self.color.unwrap_or(Rgb::new(0, 0, 0)));
    }
}
