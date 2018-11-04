// This is in-development and going to be noisey.
#![allow(dead_code)]

use std::{
    io,
};

use image;

mod geometry;
use self::geometry::Float3;

fn write_image(filename: &str) -> io::Result<()> {
    let nx = 200;
    let ny = 100;

    let mut imgbuf = image::RgbImage::new(nx, ny);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = x as f32 / nx as f32;
        let g = y as f32 / ny as f32;
        let b = 0.2_f32;

        assert!(r.abs() <= 1.0);
        assert!(g.abs() <= 1.0);
        assert!(b.abs() <= 1.0);

        let ir = (255.99 * r) as u8;
        let ig = (255.99 * g) as u8;
        let ib = (255.99 * b) as u8;
        *pixel = image::Rgb([ir, ig, ib]);
    }

    imgbuf.save(filename)?;
    Ok(())
}

fn main() {
    write_image("output.png").unwrap();
}
