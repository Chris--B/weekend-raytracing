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

fn color(ray: &Ray) -> Float3 {
    if let Some(t) = hit_sphere(&Float3::xyz(0, 0, -1), 0.5, ray) {
        let n = (ray.at_t(t) - Float3::xyz(0, 0, -1)).unit();
        0.5 * (1.0 + n)
    } else {
        // Linearly blend white and blue, depending on the "up" or
        // "downn"ness of the y coordinate.
        let white = Float3::xyz(1, 1, 1);
        let blue = Float3::xyz(0.5, 0.7, 1.0);

        let t = 0.5 * (1.0 + ray.dir.unit().y);
        Float3::lerp(t, white, blue)
    }
}

fn hit_sphere(center: &Float3, radius: Float, ray: &Ray) -> Option<Float> {
    let oc = ray.origin - *center;
    let a = ray.dir.length_sq();
    let b = 2.0 * oc.dot(&ray.dir);
    let c = oc.length_sq() - radius*radius;
    let discriminant = b*b - 4.0*a*c;

    // There are three cases to consider here:
    //      1. discriminant < 0  => There are zero real solutions, no hit.
    //      2. discriminant == 0 => There is exactly one real solutioin,
    //          and the ray just barely grazes the sphere.
    //          We'll call that a "miss"
    //      3. discriminant > 0  => There are two real solutions, so the ray
    //          intersects the sphere and we need to hande the coloring.
    if discriminant < 0.0 {
        None
    } else {
        Some((-b - discriminant.sqrt()) / (2.0 * a))
    }
}
