// This is in-development and going to be noisey.
#![allow(dead_code)]

use std::{
    io,
};

use image;

mod float3;
use self::float3::{
    Float,
    Float3
};

fn write_image(filename: &str) -> io::Result<()> {
    let nx = 200;
    let ny = 100;

    let mut imgbuf = image::RgbImage::new(nx, ny);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let mut rgb = Float3 {
            x: x as Float / nx as Float,
            y: y as Float / ny as Float,
            z: 0.2,
        };
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

fn main() {
    write_image("output.png").unwrap();
}
