use std::{
    io,
    sync::Arc,
    sync::atomic,
    time,
};

use ctrlc;
use image::{
    self,
    GenericImage,
};
use pbr;
use rayon::prelude::*;

mod camera;
mod float3;
mod hitable;
mod material;
mod math;
mod ray;

pub mod prelude;

use self::prelude::*;
use self::hitable::*;
use self::material::*;
use self::camera::*;

const MAX_RAY_RECURSION: u32 = 50;

// If something unexpected happens and all threads need to exit, this is set
// to true.
// Read it with `needs_to_exit()`.
static NEED_TO_EXIT: atomic::AtomicBool = atomic::AtomicBool::new(false);

/// A subset of our final image.
/// Tiles do not know about other tiles, but they do know their x offsets.
struct Tile {
    // x offset into the parent image
    pub offset_x: u32,
    // y-offset into the parent image
    pub offset_y: u32,
    // Pixel data for the sub image.
    // This is owned by the tile, and copied out to the parent image later.
    pub pixels: image::RgbImage,
    // A visual indicator of progress on rendering its sub image.
    pub progress: pbr::ProgressBar<pbr::Pipe>,
}

// Things can poll this method to know if they should exit early
// e.g. we received a CtrlC.
fn needs_to_exit() -> bool {
    NEED_TO_EXIT.load(atomic::Ordering::SeqCst)
}

fn main() {
    let ctrlc_handler = || NEED_TO_EXIT.store(true, atomic::Ordering::SeqCst);
    if ctrlc::set_handler(ctrlc_handler).is_err() {
        eprintln!("Unable to set Ctrl+C handler. Ctrl+C will abort the program.");
    }

    write_image("output.png").unwrap();
}

#[inline(never)]
fn write_image(filename: &str) -> io::Result<()> {
    // Number of samples per pixel.
    let ns: u32 = 4;

    // Width of the final image in pixels.
    let nx: u32 = 2 * 300;
    // Height of the final image in pixels.
    let ny: u32 = 2 * 200;

    // The count of tiles work is divided into.
    // TODO: These should be computed based on CPU cores.
    let tiles_x = 4;
    let tiles_y = 2;

    assert_eq!(nx % tiles_x, 0, "I'll solve this later");
    assert_eq!(ny % tiles_y, 0, "I'll solve this later");

    // Width of each tile in pixels.
    let tile_nx = nx / tiles_x;
    // Height of each tile in pixels.
    let tile_ny = ny / tiles_y;

    let cam = Camera::new(CameraInfo {
        lookfrom:   Float3::xyz(13., 2., 3.),
        lookat:     Float3::xyz(0., 0., 0.),
        up:         Float3::xyz(0., 1., 0.),
        vfov:       20.0,
        aspect:     nx as Float / ny as Float,
        aperature:  0.0,
        focus_dist: 20.0,
        t_start:    0.0,
        t_end:      1.0,
    });

    let mut multi_progress = pbr::MultiBar::new();

    // Each tile represents a subimage of (tile_nx, tile_ny) pixels.
    // They are combined after ray tracing.
    let mut tiles: Vec<Tile> = vec![];
    for tile_id in 0..(tiles_x * tiles_y) {
        // Tile coordinates. Must be translated into pixels with tile_n{x,y}.
        let x = tile_id % tiles_x;
        let y = tile_id / tiles_x;

        let pixels = image::RgbImage::new(tile_nx, tile_ny);

        let pixel_total = pixels.width() as u64 * pixels.height() as u64;
        let progress = multi_progress.create_bar(pixel_total);
        tiles.push(Tile {
            offset_x: x * tile_nx,
            offset_y: y * tile_ny,
            pixels,
            progress,
        });
    }

    // Load the scene
    let world = make_cover_scene();

    // Setup the progress bars
    let count = nx * ny;
    let mut running_total = 0;
    for tile in tiles.iter_mut() {
        running_total += tile.progress.total;
        tile.progress.format("[=> ]");
        tile.progress.set_max_refresh_rate(Some(time::Duration::from_millis(700)));
    }
    assert_eq!(running_total, count as u64,
               "The progress bars don't agree on how many pixels there are!");

    println!("Rendering on {} threads", rayon::current_num_threads());
    println!();

    let multi_progress_handle = std::thread::spawn(move || {
        // This blocks, so we run it on a separate thread.
        multi_progress.listen();
    });

    let before_render = time::Instant::now();
    tiles.par_iter_mut().for_each(|tile: &mut Tile| {
        'per_pixel:
        for (x, y, pixel) in tile.pixels.enumerate_pixels_mut() {
            // Adjust the (x, y) coordinates wrt our tile.
            let x = x + tile.offset_x;
            // Go through `y` "backwards"
            let y = ny - (y + tile.offset_y) + 1;

            let mut rgb = Float3::default();

            // AA through many samples.
            // We divide by `sample`, so it must not start at zero.
            for sample in 1..(ns+1) {
                let u = (x as Float + random_sfloat()) / nx as Float;
                let v = (y as Float + random_sfloat()) / ny as Float;
                let ray = cam.get_ray(u, v);

                rgb += color(&ray, &world, 0);

                // Sanity checks - no pixels are allowed outside of the range [0, 1]
                // Since we accumulate `ns` samples, each within that range,
                // the valid range at any point in the process is [0, sample].
                debug_assert!(0.0 <= rgb.x && rgb.x <= sample as Float,
                              "({}, {}) #{} rgb = {:?}",
                              x, y, sample, rgb / sample);
                debug_assert!(0.0 <= rgb.y && rgb.y <= sample as Float,
                              "({}, {}) #{} rgb = {:?}",
                              x, y, sample, rgb / sample);
                debug_assert!(0.0 <= rgb.z && rgb.z <= sample as Float,
                              "({}, {}) #{} rgb = {:?}",
                              x, y, sample, rgb / sample);
            }
            // Average samples
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

            tile.progress.inc();

            if needs_to_exit() {
                println!("Received Ctrl+C!");
                break 'per_pixel;
            }
        }
        tile.progress.finish();
    });
    let render_time = before_render.elapsed();

    match multi_progress_handle.join() {
        Ok(()) => {},
        Err(ref err) => {
            eprintln!("Error joining progress bar listener thread: {:#?}", err);
            // We ignore this error because... what else are we going to do?
        },
    }
    // Tasteful empty space.
    println!("");

    let secs = render_time.as_secs() as f64
               + render_time.subsec_millis() as f64 / 1e3;
    println!("Full scene render time: {:.3}s", secs);

    // Combine the tiles into the final image, which we write to disk.
    let mut imgbuf = image::RgbImage::new(nx, ny);
    for tile in tiles {
        let ok = imgbuf.copy_from(&tile.pixels, tile.offset_x, tile.offset_y);
        assert_eq!(ok, true,
                  concat!("imgbuf::copy_from() failed.",
                          " Is ({}, {}) out of bounds? Bounds are ({}, {})."),
                  tile.offset_x + tile.pixels.width(),
                  tile.offset_y + tile.pixels.height(),
                  imgbuf.width(),
                  imgbuf.height());
    }

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
            Float3::xyz(1., 0., 1.)
        } else {
            // If scatter hit something, but doesn't produce more rays,
            // just return the attenuation.
            attenuation.abs()
        }
    } else {
        // Linearly blend white and blue, depending on the "up" or
        // "downn"ness of the y coordinate.
        let white = Float3::xyz(1., 1., 1.);
        let blue = Float3::xyz(0.5, 0.7, 1.0);

        let t = 0.5 * (1.0 + ray.dir.unit().y);
        Float3::lerp(t, white, blue)
    }
}

#[allow(dead_code)]
fn make_green_scene() -> HitableList {
    HitableList {
        hitables: vec![
            Box::new(Sphere {
                center: Float3::xyz(0., 0., -1.),
                radius: 0.5,
                material: Arc::new(Lambertian {
                    albedo: Float3::xyz(0.1, 0.2, 0.5),
                }),
            }),
            Box::new(Sphere {
                center: Float3::xyz(0.0, -100.5, -1.0),
                radius: 100.0,
                material: Arc::new(Lambertian {
                    albedo: Float3::xyz(0.8, 0.8, 0.0),
                }),
            }),
            Box::new(Sphere {
                center: Float3::xyz(1., 0., -1.),
                radius: 0.5,
                material: Arc::new(Metal {
                    albedo: Float3::xyz(0.8, 0.6, 0.2),
                    fuzz:   0.0,
                }),
            }),
            Box::new(Sphere {
                center: Float3::xyz(-1., 0., -1.),
                radius: 0.5,
                material: Arc::new(Dielectric {
                    refraction_index: 1.5,
                }),
            }),
            Box::new(Sphere {
                center: Float3::xyz(-1., 0., -1.),
                radius: -0.45, // Negative radius makes the above sphere hollow.
                material: Arc::new(Dielectric {
                    refraction_index: 1.5,
                }),
            }),
        ],
    }
}

fn make_cover_scene() -> HitableList {
    // A giant, darkish colored sphere to act as the floor.
    let ground: Box<dyn Hitable> = Box::new(Sphere {
        center: Float3::xyz(0., -1000., 0.),
        radius: 1000.0,
        material: Arc::new(Lambertian {
            albedo: Float3::xxx(0.5),
        })
    });

    let mut spheres = vec![];
    spheres.push(ground);

    // This material can be reused, since its parameters don't change
    // between spheres.
    let dielectric = Arc::new(Dielectric {
        refraction_index: 1.5,
    });

    // This material is colored by its surface normal and nothing else.
    // It does not refract, reflect, or change within its environment.
    let _normal_map_material = Arc::new(NormalToRgb {});

    let point = Float3::xyz(4.0, 0.2, 0.0);
    let radius = 0.2;
    const GRID: Float = 1.0;

    // Many, many little spheres.
    for a in -10..10 {
        let a = a as Float;
        for b in -10..10 {
            let b = b as Float;

            // These are positive random floats to avoid collisions.
            let center = Float3 {
                x: a + (GRID - radius) * random_float(),
                y: radius,
                z: b + (GRID - radius) * random_float(),
            };

            if (center - point).length_sq() > (0.9*0.9) {
                let sphere: Box<dyn Hitable>;
                sphere = match random_float() {
                    // Diffuse
                    prob if prob < 0.8 => {
                        Box::new(MovingSphere {
                            sphere: Sphere {
                                center,
                                radius,
                                material: Arc::new(Lambertian {
                                    albedo: Float3 {
                                        x: random_float() * random_float(),
                                        y: random_float() * random_float(),
                                        z: random_float() * random_float(),
                                    },
                                }),
                            },
                            // Only Lambertian spheres bounce
                            motion: Float3 {
                                x: 0.0,
                                y: 0.5 * random_float(),
                                z: 0.0,
                            },
                        })
                    }
                    // Metal
                    prob if prob < 0.90 => {
                        Box::new(MovingSphere {
                            sphere: Sphere {
                                center,
                                radius,
                                material: Arc::new(Metal {
                                    albedo: Float3 {
                                        x: random_float(),
                                        y: random_float(),
                                        z: random_float(),
                                    },
                                    fuzz: 0.5 * random_float(),
                                }),
                            },
                            // Stationary
                            motion: Float3::new(),
                        })
                    }
                    // Glass
                    _ => {
                        Box::new(MovingSphere {
                            sphere: Sphere {
                                center,
                                radius,
                                material: dielectric.clone(),
                            },
                            // Stationary - the glass would break!
                            motion: Float3::new(),
                        })
                    }
                };
                spheres.push(sphere);
            }
        }
    }

    // Three big spheres
    spheres.push(Box::new(Sphere {
        center:   Float3::xyz(0., 1., 0.),
        radius:   1.,
        material: dielectric.clone(),
    }));

    spheres.push(Box::new(Sphere {
        center:   Float3::xyz(-4., 1., 0.),
        radius:   1.,
        material: Arc::new(Lambertian {
            albedo: Float3::xyz(0.4, 0.2, 0.1),
        }),
    }));

    spheres.push(Box::new(Sphere {
        center:   Float3::xyz(4., 1., 0.),
        radius:   1.,
        material: Arc::new(Metal {
            albedo: Float3::xyz(0.7, 0.6, 0.5),
            fuzz:   0.,
        }),
    }));

    HitableList { hitables: spheres }
}
