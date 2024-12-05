use crate::point::Point;
use fax::decoder;
use fax::Color;
use image::{DynamicImage, GrayImage, ImageReader, Luma, RgbImage};
use imageproc::drawing;
use std::path::Path;

/// This is the computation of the Kapur level using equ. (18) in
/// [the original article].
/// [the original article]: https://doi.org/10.1016/0734-189X(85)90125-2
fn kapur_level(img: &GrayImage) -> u8 {
    let hist = imageproc::stats::histogram(img);
    let histogram = &hist.channels[0]; // GrayImage has only one channel

    let total_pixels = (img.width() * img.height()) as f64;

    // The p_i in the article. They describe the probability of encountering
    // gray-level i.
    let mut p = [0.0f64; 256];
    for i in 0..=255 {
        p[i] = histogram[i] as f64 / total_pixels;
    }

    // The P_s in the article, which is the probability of encountering
    // gray-level <= s.
    let mut cum_p = [0.0f64; 256];
    cum_p[0] = p[0];
    for i in 1..=255 {
        cum_p[i] = cum_p[i - 1] + p[i];
    }

    // The H_s in the article. These are the entropies attached to the
    // distributions p[0],...,p[s].
    let mut h = [0.0f64; 256];
    if p[0] > f64::EPSILON {
        h[0] = -p[0] * p[0].ln();
    }
    for s in 1..=255 {
        if p[s] > f64::EPSILON {
            h[s] = h[s - 1] - p[s] * p[s].ln();
        } else {
            h[s] = h[s - 1]
        }
    }

    let mut max_entropy = f64::MIN;
    let mut best_threshold = usize::MIN;

    for s in 0..=255 {
        // psi_s is the total entropy of foreground and background at threshold
        // level s. Instead of computing them separately, equation (18) in the
        // article, which simplifies this to this:
        let psi_s = (cum_p[s] * (1.0 - cum_p[s])).ln()
            + h[s] / cum_p[s]
            + (h[255] - h[s]) / (1.0 - cum_p[s]);

        if psi_s > max_entropy {
            max_entropy = psi_s;
            best_threshold = s;
        }
    }

    best_threshold as u8
}
pub fn fax_to_grayimage(data: &[u8], width: u32, height: u32) -> GrayImage {
    let mut result = GrayImage::new(width, height);
    let mut y = 0;
    decoder::decode_g4(data.iter().cloned(), width as u16, None, |transitions| {
        for (x, c) in decoder::pels(transitions, width as u16).enumerate() {
            let pixel = match c {
                Color::Black => Luma([0u8]),
                Color::White => Luma([255u8]),
            };
            result.put_pixel(x as u32, y, pixel);
        }
        y += 1;
    });

    result
}
pub fn binary_image_from_image(img: DynamicImage) -> GrayImage {
    let res = img.into_luma8();
    let threshold = kapur_level(&res);

    imageproc::contrast::threshold(&res, threshold, imageproc::contrast::ThresholdType::Binary)
}

pub fn binary_image_from_file(path: &String) -> GrayImage {
    let image_path = Path::new(path);
    let img = ImageReader::open(image_path)
        .expect("failed to open file")
        .decode()
        .expect("failed to decode image");

    binary_image_from_image(img)
}
pub fn gray_to_rgb(gray_image: &GrayImage) -> RgbImage {
    let (width, height) = gray_image.dimensions();
    let mut rgb_image = RgbImage::new(width, height);

    for (x, y, gray_pixel) in gray_image.enumerate_pixels() {
        let intensity = gray_pixel[0];
        rgb_image.put_pixel(x, y, image::Rgb([intensity, intensity, intensity]));
    }

    rgb_image
}

pub fn draw_rectangle_around_box(
    img: &mut RgbImage,
    topleft: Point,
    botright: Point,
    color: image::Rgb<u8>,
) {
    let strength = 4;
    let x = topleft.x as i32;
    let y = topleft.y as i32;
    let size_x = botright.x - topleft.x;
    let size_y = botright.y - topleft.y;

    for i in 0..strength {
        drawing::draw_hollow_rect_mut(
            img,
            imageproc::rect::Rect::at(x - i, y - i)
                .of_size(size_x + 2 * i as u32, size_y + 2 * i as u32),
            color,
        );
    }
}

pub fn draw_circle_around_box(
    img: &mut RgbImage,
    topleft: Point,
    botright: Point,
    color: image::Rgb<u8>,
) {
    let radius = ((botright.x - topleft.x) / 3) as i32;
    let center = Point {
        x: (topleft.x + botright.x) / 2,
        y: (topleft.y + botright.y) / 2,
    };

    for i in 0..(radius / 4) {
        drawing::draw_hollow_circle_mut(img, (center.x as i32, center.y as i32), radius + i, color);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn rgb_to_egui_color_image(image: &RgbImage) -> egui::ColorImage {
    let (width, height) = image.dimensions();
    let pixels: Vec<egui::Color32> = image
        .pixels()
        .map(|p| egui::Color32::from_rgb(p[0], p[1], p[2]))
        .collect();

    egui::ColorImage {
        size: [width as usize, height as usize],
        pixels,
    }
}
