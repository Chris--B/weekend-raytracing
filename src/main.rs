// This is in-development and going to be noisey.
#![allow(dead_code)]

use std::{
    io,
};

use image;

mod float3;
mod ray;

use self::float3::{
    Float,
    Float3
};
use self::ray::Ray;

fn main() {
    write_image("output.png").unwrap();
}

fn write_image(filename: &str) -> io::Result<()> {
    let nx = 200;
    let ny = 100;

    let lower_left = Float3::xyz(-2, -1, -1);
    let horizontal = Float3::xyz(4, 0, 0);
    let vertical   = Float3::xyz(0, 2, 0);
    let origin     = Float3::xyz(0, 0, 0);

    let mut imgbuf = image::RgbImage::new(nx, ny);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        // Go through `y` "backwards"
        let y = ny - y + 1;
        let u = x as Float / nx as Float;
        let v = y as Float / ny as Float;
        let ray = Ray {
            origin,
            dir: lower_left + u*horizontal + v*vertical,
        };
        let mut rgb = color(&ray);
        assert!(rgb.x.abs() <= 1.0);
        assert!(rgb.y.abs() <= 1.0);
        assert!(rgb.z.abs() <= 1.0);
        rgb *= 255.99;

        *pixel = image::Rgb([
            rgb.x as u8,
            rgb.y as u8,
            rgb.z as u8,
        ]);
    }

    imgbuf.save(filename)?;
    Ok(())
}

// Linearly blend white and blue, depending on the "up" or
// "downn"ness of the y coordinate.
fn color(ray: &Ray) -> Float3 {
    let white = Float3::xyz(1, 1, 1);
    let blue = Float3::xyz(0.5, 0.7, 1.0);

    let unit_dir = ray.dir.unit();
    // Scale [-1, 1] to [0, 1]
    let t = 0.5 * (unit_dir.y + 1.0);
    Float3::lerp(t, white, blue)
}
