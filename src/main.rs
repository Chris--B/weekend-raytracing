// This is in-development and going to be noisey.
#![allow(dead_code)]

use std::{
    io,
    rc::Rc,
    time,
};

use image;
use rand::prelude::*;
use pbr::ProgressBar;

mod camera;
mod float3;
mod hitable;
mod material;
mod ray;

use self::camera::Camera;
use self::float3::*;
use self::hitable::*;
use self::material::*;
use self::ray::Ray;

fn main() {
    write_image("output.png").unwrap();
}

fn write_image(filename: &str) -> io::Result<()> {
    let nx: u32 = 400;
    // Our camera is dumb and won't fix our aspect ratio.
    // It currently assumes 2:1.
    let ny: u32 = nx / 2;
    let ns: u32 = 200;

    let cam = Camera::default();
    let world = HitableList {
        hitables: vec![
            Box::new(Sphere {
                center: Float3::xyz(0, 0, -1),
                radius: 0.5,
                material: Rc::new(Lambertian {
                    albedo: Float3::xyz(0.1, 0.2, 0.5),
                }),
            }),
            Box::new(Sphere {
                center: Float3::xyz(0.0, -100.5, -1.0),
                radius: 100.0,
                material: Rc::new(Lambertian {
                    albedo: Float3::xyz(0.8, 0.8, 0.0),
                }),
            }),
            Box::new(Sphere {
                center: Float3::xyz(1, 0, -1),
                radius: 0.5,
                material: Rc::new(Metal {
                    albedo: Float3::xyz(0.8, 0.6, 0.2),
                    fuzz:   0.0,
                }),
            }),
            Box::new(Sphere {
                center: Float3::xyz(-1, 0, -1),
                radius: 0.5,
                material: Rc::new(Dielectric {
                    refraction_index: 1.5,
                }),
            }),
            Box::new(Sphere {
                center: Float3::xyz(-1, 0, -1),
                radius: -0.45, // Negative radius makes the above sphere hollow.
                material: Rc::new(Dielectric {
                    refraction_index: 1.5,
                }),
            }),
        ],
    };

    let count = nx * ny;
    let mut progress = ProgressBar::new(count as u64);
    progress.format("[=> ]");
    progress.set_max_refresh_rate(Some(time::Duration::from_millis(700)));

    let mut imgbuf = image::RgbImage::new(nx, ny);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        // Go through `y` "backwards"
        let y = ny - y + 1;
        let mut rgb = Float3::default();
        // MSAA
        for sample in 0..ns {
            let u = (x as Float + random::<Float>()) / nx as Float;
            let v = (y as Float + random::<Float>()) / ny as Float;
            let ray = cam.get_ray(u, v);
            rgb += color(&ray, &world, 0);
            // Sanity checks
            // TODO: These are probably expensive, this is a hot loop.
            assert!(0.0 <= rgb.x && rgb.x <= ns as Float,
                    "({}, {}) #{} rgb = {:?}", x, y, sample, rgb / sample);
            assert!(0.0 <= rgb.y && rgb.y <= ns as Float,
                    "({}, {}) #{} rgb = {:?}", x, y, sample, rgb / sample);
            assert!(0.0 <= rgb.z && rgb.z <= ns as Float,
                    "({}, {}) #{} rgb = {:?}", x, y, sample, rgb / sample);
        }
        // Average mutlisamples
        rgb /= ns;
        // Gamma correct
        rgb = rgb.sqrt();
        // Scale into u8 range
        rgb *= 255.99;
        *pixel = image::Rgb([
            rgb.x as u8,
            rgb.y as u8,
            rgb.z as u8,
        ]);

        // Hopefully this isn't expensive.
        progress.inc();
    }
    // The progress bar is only updated periodically. If it finished counting
    // before it was due for another refresh, it won't update.
    // We force it to finish here.
    progress.finish_println("\n");

    imgbuf.save(filename)?;
    Ok(())
}

fn color(ray: &Ray, world: &dyn Hitable, depth: u32) -> Float3 {
    if let Some(hit_record) = world.hit(ray, 1.0e-3, std::f64::MAX as Float) {
        let mut scattered = Ray::default();
        let mut attenuation = Float3::new();
        if depth < 50 &&
           hit_record.material.scatter(ray,
                                       &hit_record,
                                       &mut attenuation,
                                       &mut scattered)
        {
            attenuation * color(&scattered, world, depth + 1)
        } else {
            Float3::new()
        }
    } else {
        // Linearly blend white and blue, depending on the "up" or
        // "downn"ness of the y coordinate.
        let white = Float3::xyz(1, 1, 1);
        let blue = Float3::xyz(0.5, 0.7, 1.0);

        let t = 0.5 * (1.0 + ray.dir.unit().y);
        Float3::lerp(t, white, blue)
    }
}
