#![allow(dead_code)]

use std::{
    collections::hash_map,
    hash::{
        self,
        Hasher,
    },
    mem,
    path,
    sync::Arc,
    sync::atomic,
    time,
};

use ctrlc;
use image::{
    GenericImage,
};
use pbr;

use rand::prelude::*;
use rayon::prelude::*;
use structopt::*;

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

#[derive(Debug, StructOpt)]
#[structopt(name="raytracer",
            about="Traces rays",
            raw(
                setting="clap::AppSettings::DeriveDisplayOrder",
                setting="clap::AppSettings::DisableVersion"))]
struct Opt {
    // ===== Options ==========

    /// Width of image in pixels
    #[structopt(default_value="800", short, long)]
    width: u32,

    /// Height of image in pixels
    #[structopt(default_value="600", short, long)]
    height: u32,

    /// Number of rays cast per pixel
    #[structopt(default_value="2", short, long)]
    samples_per_pixel: u32,

    /// Number of tiles to subdivide the image into
    // TODO: Pick this automatically and default to "0"
    #[structopt(default_value="16", short, long)]
    tiles: u32,

    /// Number of threads used in `rayon`'s thread pool.
    /// 0 uses system default
    #[structopt(default_value="0", short, long)]
    jobs: u8, // Like we're going to run on 256-thread machines.

    /// File to write image data into
    // It will be created if it does not exist, and overwriten if it does
    // Note that this extension is how `image` determines encoding
    #[structopt(default_value="output.png", parse(from_os_str), short, long)]
    output: path::PathBuf,

    /// Vertical field of view
    #[structopt(default_value="20.0", long)]
    vfov: Float,

    /// Camera aperature
    #[structopt(default_value="0.1", short, long)]
    aperature: Float,

    /// Camera focus point
    #[structopt(default_value="10.0", short, long)]
    focus_dist: Float,

    /// Time of initial exposure
    #[structopt(default_value="0.0", long="t-start")]
    t_start: Float,

    /// Time of final exposure
    #[structopt(default_value="0.5", long="t-end")]
    t_end: Float,

    /// Select a scene to render.
    /// NOT IMPLEMENTED
    #[structopt(default_value="cover", long)]
    scene: String,

    // ===== Flags ==========

    /// Enable more detailed output
    #[structopt(short, long)]
    verbose: bool,

    /// Start the renderer in interactive mode.
    /// NOT IMPLEMENTED
    #[structopt(short, long)]
    interactive: bool,

    /// Skip some tiles in a checkerboard fashion. Useful for debugging tiles
    #[structopt(long="checkerboard-tiles")]
    checkerboard_tiles: bool,
}

/// A subset of our final image.
/// Tiles do not know about other tiles, but they do know their x offsets.
struct Tile {
    /// Unique id for each tile
    pub tile_id: u32,

    /// x coordinate of tile, in the tile grid
    pub tile_x: u32,

    /// y coordinate of tile, in the tile grid
    pub tile_y: u32,

    /// x offset into the parent image
    pub offset_x: u32,

    /// y-offset into the parent image
    pub offset_y: u32,

    /// Pixel data for the sub image
    /// This is owned by the tile, and copied out to the parent image later.
    pub pixels: image::RgbImage,

    /// A visual indicator of progress made rendering this tile
    pub progress: pbr::ProgressBar<pbr::Pipe>,
}

// Tasks use this to exit early
static NEED_TO_EXIT: atomic::AtomicBool = atomic::AtomicBool::new(false);

// Things can poll this method to know if they should exit early
// e.g. we received a CtrlC.
fn needs_to_exit() -> bool {
    NEED_TO_EXIT.load(atomic::Ordering::SeqCst)
}

// Things can call this method to signal that the application should exit
// Calling this multiple times is fine but redundant.
fn signal_exit() {
    NEED_TO_EXIT.store(true, atomic::Ordering::SeqCst);
}

fn hash_it(thing: &impl hash::Hash) -> u64 {
    let mut hasher = hash_map::DefaultHasher::new();
    thing.hash(&mut hasher);
    hasher.finish()
}

fn pick_tiling_dimensions(n_tiles: u32, nx: u32, ny: u32) -> (u32, u32) {
    let aspect: Float = (nx as Float) / (ny as Float);

    // We want to create roughly square tiles, but they need to divide the
    // image's width exactly.
    // In the case of a square image (W == H), we could just call `.sqrt()`.
    // More generally, we need to scale the number of tiles along one side
    // by the aspect ratio (W/H).
    // Here's the problem described in formula.
    //          x := # of tiles along thex axis (width)
    //          y := # of tiles along they axis (height)
    //      x     == ASPECT * y;
    //      x * y == n_tiles;
    // Since we know `ASPECT` and `n_tiles`, we re-arrange the above as:
    //      x    = ASPECT * y
    //      y**2 = n_tiles / ASPECT
    // This is enough to compute the value and round it to an integer.
    let raw_y: f64 = (n_tiles as Float / aspect).sqrt().round() as f64;

    // At this point `raw_y` is a float and might not divide the requested
    // tile count easily. We need to decide wether to opt for more square
    // tiles by disregarding the requested tile count, or opt for hitting
    // the tile count but with less square tiles.
    // We opt for respecting the requested tile count.
    // We do this by rounding the previous raw_y value to the closest factor
    // of the tile count.
    let mut best_factor = 1;                // First factor
    let mut best_error = n_tiles as f64;  // Worst possible error
    for factor in math::factors(n_tiles) {
        // We want to minimize "error". Here, error is defined as the
        // ratio from our raw, ideal y with the factor in question.
        // If the factor is *smaller*, we flip the ratio to allow this
        // process to shorten the height of tiles, if need be.
        let mut next_error = raw_y / factor as f64;
        if next_error < 1.0 {
            next_error = 1.0 / next_error;
        }

        if next_error < best_error {
            best_factor = factor;
            best_error = next_error;
        }
    }
    let y = best_factor;
    let x = n_tiles / y;
    assert_eq!(n_tiles as f64 / y as f64, x as f64,
               "Tile size calculation should be exact, integer math!");
    (x, y)
}

fn main() {
    // Parse CLI
    let opt = Opt::from_args();

    // If the user uses Ctrl+C to quit early, we want to handle that.
    // Specifically, we write what image data has been generated to disk.
    if ctrlc::set_handler(signal_exit).is_err() {
        eprintln!("Unable to set Ctrl+C handler. Ctrl+C will abort the program.");
    }

    // Bulk of the work
    let imgbuf = write_image(&opt);

    imgbuf.save(&opt.output).unwrap();
    if let Ok(path) = opt.output.canonicalize() {
        println!("Successfully wrote out to {}", path.display());
    }
}

fn write_image(opt: &Opt) -> image::RgbImage {
    let ns: u32 = opt.samples_per_pixel;
    let nx: u32 = opt.width;
    let ny: u32 = opt.height;

    let (tiles_x, tiles_y) = pick_tiling_dimensions(opt.tiles, nx, ny);

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
        vfov:       opt.vfov,
        aspect:     nx as Float / ny as Float,
        aperature:  opt.aperature,
        focus_dist: opt.focus_dist,
        t_start:    opt.t_start,
        t_end:      opt.t_end,
    });

    let mut multi_progress = pbr::MultiBar::new();

    // Each tile represents a subimage of (tile_nx, tile_ny) pixels.
    // They are combined after ray tracing.
    let mut tiles: Vec<Tile> = vec![];
    for tile_id in 0..(tiles_x * tiles_y) {
        // Tile coordinates. Must be translated into pixels with tile_n{x,y}.
        let x = tile_id % tiles_x;
        let y = tile_id / tiles_x;

        // When we're set to skip tiles, don't even enqueue it.
        if opt.checkerboard_tiles {
            if x % 2 != y % 2 {
                continue;
            }
        }

        let pixels = image::RgbImage::new(tile_nx, tile_ny);
        let pixel_total = pixels.width() as u64 * pixels.height() as u64;

        let mut progress = multi_progress.create_bar(pixel_total);
        progress.message(&format!("Tile {:>2} ({}, {}): ", tile_id, x, y));
        progress.format("[=> ]");
        progress.set_max_refresh_rate(Some(time::Duration::from_millis(700)));

        tiles.push(Tile {
            tile_id,
            tile_x: x,
            tile_y: y,
            offset_x: x * tile_nx,
            offset_y: y * tile_ny,
            pixels,
            progress,
        });
    }

    // Load the scene
    let world = make_cover_scene();

    // Sanity check the progress bars.
    // If we're doing checkboarded tiles, we don't care since it would
    // fail anyway.
    if !opt.checkerboard_tiles {
        let pb_count: u64 = tiles.iter().map(|t| t.progress.total).sum();
        let px_count: u64 = (nx * ny) as u64;
        assert_eq!(pb_count, px_count,
                "The progress bars don't agree on how many pixels there are!");
    }

    rayon::ThreadPoolBuilder::new()
        .num_threads(opt.jobs as usize)
        .build_global()
        .expect("Unexpected failure with rayon::ThreadPoolBuilder");
    eprintln!("Rendering on {} threads\n", rayon::current_num_threads());

    let h_listener = std::thread::spawn(move || {
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
                break 'per_pixel;
            }
        }
        tile.progress.finish();
    });
    let render_time = before_render.elapsed();

    match h_listener.join() {
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
    eprintln!("Full scene render time: {:.3}s", secs);

    // Combine the tiles into the final image, which we write to disk.
    let mut imgbuf = image::RgbImage::new(nx, ny);
    for tile in tiles {
        let ok = imgbuf.copy_from(&tile.pixels, tile.offset_x, tile.offset_y);
        assert_eq!(ok, true,
                  concat!("imgbuf::copy_from() failed. ",
                          "Is ({}, {}) out of bounds? Bounds are ({}, {})."),
                  tile.offset_x + tile.pixels.width(),
                  tile.offset_y + tile.pixels.height(),
                  imgbuf.width(),
                  imgbuf.height());
    }

    imgbuf
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
    // Sigh... All of this to hash two strings into 128-bits. ._.
    //
    // `mem::transmute` is unsafe in general because many types don't appreciate
    // arbitary bit patterns being operated on like they're that type.
    // We're transmuting from two primitives types, so this is fine.
    // All possible 128-bit patterns for [u8; 16] are valid here.
    let hash_bytes: [u8; 16] = unsafe {
        // TODO: When `u64::to_be_bytes` and friends stabilize, we can use those.
        //       See https://github.com/rust-lang/rust/issues/52963
        // The advantage of that will be consistency across endian platforms.
        mem::transmute([
            hash_it(b"Katy's Penguin"),
            hash_it(b"Alyssa's Panda"),
        ])
    };
    let mut rng = SmallRng::from_seed(hash_bytes);

    // Our accelaration structure is a list of spheres.
    let mut spheres: Vec<Box<dyn Hitable>> = vec![];

    // A giant, darkish colored sphere to act as the floor.
    let ground = Box::new(Sphere {
        center: Float3::xyz(0., -1000., 0.),
        radius: 1000.0,
        material: Arc::new(Lambertian {
            albedo: Float3::xxx(0.5),
        })
    });
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
                x: a + (GRID - radius) * rng.gen::<Float>(),
                y: radius,
                z: b + (GRID - radius) * rng.gen::<Float>(),
            };

            if (center - point).length_sq() > (0.9*0.9) {
                let sphere: Box<dyn Hitable>;
                sphere = match rng.gen::<Float>() {
                    // Diffuse
                    prob if prob < 0.7 => {
                        Box::new(MovingSphere {
                            sphere: Sphere {
                                center,
                                radius,
                                material: Arc::new(Lambertian {
                                    albedo: Float3 {
                                        x: rng.gen::<Float>() * rng.gen::<Float>(),
                                        y: rng.gen::<Float>() * rng.gen::<Float>(),
                                        z: rng.gen::<Float>() * rng.gen::<Float>(),
                                    },
                                }),
                            },
                            // Only Lambertian spheres bounce
                            motion: Float3 {
                                x: 0.0,
                                y: 0.5 * rng.gen::<Float>(),
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
                                        x: rng.gen::<Float>(),
                                        y: rng.gen::<Float>(),
                                        z: rng.gen::<Float>(),
                                    },
                                    fuzz: 0.5 * rng.gen::<Float>(),
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
