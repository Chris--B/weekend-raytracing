use std::{
    io,
    rc::Rc,
    time,
};

use image;
use pbr::ProgressBar;

mod camera;
mod float3;
mod hitable;
mod material;
mod math;
mod ray;

use self::camera::*;
use self::float3::*;
use self::hitable::*;
use self::material::*;
use self::math::*;
use self::ray::Ray;

const MAX_RAY_RECURSION: u32 = 50;

fn main() {
    write_image("output.png").unwrap();
}

fn write_image(filename: &str) -> io::Result<()> {
    let nx: u32 = 300;
    let ny: u32 = 200;
    let ns: u32 = 8;

    let cam = Camera::new(CameraInfo {
        lookfrom:   Float3::xyz(13., 2., 3.),
        lookat:     Float3::xyz(0, 0, 0),
        up:         Float3::xyz(0, 1, 0),
        vfov:       20.0,
        aspect:     nx as Float / ny as Float,
        aperature:  0.1,
        focus_dist: 20.0,
    });
    let world = make_cover_scene();

    let count = nx * ny * ns;
    let mut progress = ProgressBar::new(count as u64);
    progress.format("[=> ]");
    progress.set_max_refresh_rate(Some(time::Duration::from_millis(700)));

    let mut imgbuf = image::RgbImage::new(nx, ny);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        // Go through `y` "backwards"
        let y = ny - y + 1;
        let mut rgb = Float3::default();
        // MSAA
        for sample in 1..(ns+1) {
            let u = (x as Float + random_sfloat()) / nx as Float;
            let v = (y as Float + random_sfloat()) / ny as Float;
            let ray = cam.get_ray(u, v);

            rgb += color(&ray, &world, 0);

            // Sanity checks - no pixels are allowed outside of the range [0, 1]
            // Since we accumulate `ns` samples, each within that range,
            // the valid range at any point in the process is [0, sample].
            debug_assert!(0.0 <= rgb.x && rgb.x <= sample as Float,
                          "({}, {}) #{} rgb = {:?}", x, y, sample, rgb / sample);
            debug_assert!(0.0 <= rgb.y && rgb.y <= sample as Float,
                          "({}, {}) #{} rgb = {:?}", x, y, sample, rgb / sample);
            debug_assert!(0.0 <= rgb.z && rgb.z <= sample as Float,
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
        progress.add(ns as u64);
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
        if depth < MAX_RAY_RECURSION &&
           hit_record.material.scatter(ray,
                                       &hit_record,
                                       &mut attenuation,
                                       &mut scattered)
        {
            attenuation * color(&scattered, world, depth + 1)
        } else if depth == MAX_RAY_RECURSION {
            Float3::xyz(1, 0, 1)
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

fn make_green_scene() -> HitableList {
    HitableList {
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
    }
}

fn make_cover_scene() -> HitableList {
    // A giant, darkish colored sphere to act as the floor.
    let ground: Box<dyn Hitable> = Box::new(Sphere {
        center: Float3::xyz(0, -1000, 0),
        radius: 1000.0,
        material: Rc::new(Lambertian {
            albedo: Float3::xxx(0.5),
        })
    });

    let mut spheres = vec![];
    spheres.push(ground);

    // This material can be reused, since its parameters don't change
    // between spheres.
    let dielectric = Rc::new(Dielectric {
        refraction_index: 1.5,
    });

    let point = Float3::xyz(4.0, 0.2, 0.0);
    let radius = 0.2;
    const GRID: Float = 1.0;

    // Many, many little spheres.
    for a in -11..11 {
        let a = a as Float;
        for b in -11..11 {
            let b = b as Float;

            // These are positive random floats to avoid collisions.
            let center = Float3 {
                x: a + (GRID - radius) * random_float(),
                y: radius,
                z: b + (GRID - radius) * random_float(),
            };

            if (center - point).length_sq() > (0.9*0.9) {
                let sphere: Box<dyn Hitable>;
                sphere = match random_sfloat() {
                    // Diffuse
                    prob if prob < 0.8 => {
                        Box::new(Sphere {
                            center,
                            radius,
                            material: Rc::new(Lambertian {
                                albedo: Float3 {
                                    x: random_float() * random_float(),
                                    y: random_float() * random_float(),
                                    z: random_float() * random_float(),
                                },
                            }),
                        })
                    },
                    // Metal
                    prob if prob < 0.95 => {
                        Box::new(Sphere {
                            center,
                            radius,
                            material: Rc::new(Metal {
                                albedo: Float3 {
                                    x: random_float(),
                                    y: random_float(),
                                    z: random_float(),
                                },
                                fuzz: 0.5 * random_float(),
                            }),
                        })
                    },
                    // Glass
                    _ => {
                        Box::new(Sphere {
                            center,
                            radius,
                            material: dielectric.clone(),
                        })
                    },
                };
                spheres.push(sphere);
            }
        }
    }

    // Three big speheres
    spheres.push(Box::new(Sphere {
        center:   Float3::xyz(0., 1., 0.),
        radius:   1.,
        material: dielectric.clone(),
    }));

    spheres.push(Box::new(Sphere {
        center:   Float3::xyz(-4., 1., 0.),
        radius:   1.,
        material: Rc::new(Lambertian {
            albedo: Float3::xyz(0.4, 0.2, 0.1),
        }),
    }));

    spheres.push(Box::new(Sphere {
        center:   Float3::xyz(4., 1., 0.),
        radius:   1.,
        material: Rc::new(Metal {
            albedo: Float3::xyz(0.7, 0.6, 0.5),
            fuzz:   0.,
        }),
    }));

    HitableList { hitables: spheres }
}
