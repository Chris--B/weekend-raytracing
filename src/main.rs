// This is in-development and going to be noisey.
#![allow(dead_code)]

use std::{
    io,
};

use image;

mod float3;
mod ray;
mod hitable;

use self::float3::{
    Float,
    Float3
};
use self::ray::Ray;
use self::hitable::{
    Hitable,
    HitableList,
    Sphere,
};

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

    let world = HitableList {
        hitables: vec![
            Box::new(Sphere {
                center: Float3::xyz(0, 0, -1),
                radius: 0.5
            }),
            Box::new(Sphere {
                center: Float3::xyz(0.0, -100.5, -1.0),
                radius: 100.0
            }),
        ],
    };

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
        let mut rgb = color(&ray, &world);
        assert!(0.0 <= rgb.x && rgb.x <= 1.0);
        assert!(0.0 <= rgb.y && rgb.y <= 1.0);
        assert!(0.0 <= rgb.z && rgb.z <= 1.0);
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

fn color(ray: &Ray, world: &dyn Hitable) -> Float3 {
    if let Some(hit_record) = world.hit(ray, 0.0, std::f64::MAX as Float) {
        0.5 * (1.0 + hit_record.normal)
    } else {
        // Linearly blend white and blue, depending on the "up" or
        // "downn"ness of the y coordinate.
        let white = Float3::xyz(1, 1, 1);
        let blue = Float3::xyz(0.5, 0.7, 1.0);

        let t = 0.5 * (1.0 + ray.dir.unit().y);
        Float3::lerp(t, white, blue)
    }
}
