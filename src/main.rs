
use std::{
    io,
};

fn write_p3(writer: &mut impl io::Write) -> io::Result<()> {
    let nx = 200;
    let ny = 100;
    writeln!(writer, "P3")?;
    writeln!(writer, "{} {}", nx, ny)?;
    writeln!(writer, "255")?;

    for j in (0..ny).rev() {
        let j = j as f32;
        for i in 0..nx {
            let i = i as f32;
            let r = i / nx as f32;
            let g = j / ny as f32;
            let b = 0.2;

            let ir = (255.99 * r) as u32;
            let ig = (255.99 * g) as u32;
            let ib = (255.99 * b) as u32;

            writeln!(writer, "{} {} {}", ir, ig, ib)?;
        }
    }

    Ok(())
}

fn main() {
    let mut stdout = std::io::stdout();
    write_p3(&mut stdout).unwrap();
}
