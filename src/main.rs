extern crate starfield_render;
extern crate clap;
extern crate stl;
extern crate nalgebra;
#[cfg(feature="termite")]
extern crate termite;

use std::fs::File;
use std::path::Path;
use starfield_render as sf;
use clap::{App, Arg};
use nalgebra::{Vector3, Vector4, Rotation3, Rotation, Rotate};
use std::io::{Write,stdout};
use std::f32;
use std::f32::consts::PI;

#[cfg(feature="termite")]
fn get_defaults() -> (usize, usize, f32)
{
    let (width, height) = termite::get_term_dims();
    let char_aspect = termite::char_aspect();
    (width, height, char_aspect)
}

#[cfg(not(feature="termite"))]
fn get_defaults() -> (usize, usize, f32)
{
    (40, 20, 0.5)
}

fn print_mat(buf: &sf::DepthBuffer<sf::Pixel>)
{
    for y in (0..buf.height).rev() {
        for x in 0..buf.width {
            match buf.get(x,y) {
                &Some((ref col, _)) => print!("\x1B[48;5;{}m ", sf::to_256_color(col, x, y)),
                &None => print!("\x1B[48;5;0m ")
            }
        }
        if y != 0 {
            println!("");
        }
    }
}

enum Mode {
    Static,
    Rotation
}

fn main() {
    let matches = App::new("stlview")
                        .about("Displays stl files to the terminal.")
                        .arg(Arg::with_name("file")
                             .required(true)
                             .value_name("FILE")
                             .help("The path to the stl file to display"))
                        .arg(Arg::with_name("width")
                             .short("w")
                             .long("width")
                             .value_name("WIDTH")
                             .help("The width of the viewport in characters."))
                        .arg(Arg::with_name("height")
                             .short("h")
                             .long("height")
                             .value_name("HEIGHT")
                             .help("The height of the viewport in characters. (1/2 of width is usually a good bet)"))
                        .arg(Arg::with_name("up")
                             .short("u")
                             .long("up")
                             .value_name("DIRECTION")
                             .help("Axis to use as 'up' direction. Defaults to z."))
                        .arg(Arg::with_name("mode")
                             .short("m")
                             .long("mode")
                             .value_name("MODE")
                             .help("One of static (short form s) or rotation (r). static simply prints the render, while rotation animates."))
                        .get_matches();

    let filename = matches.value_of("file").unwrap();

    let (width, height, char_aspect) = get_defaults();

    let (width, height) = if width as f32 > height as f32 * char_aspect{
        ((height as f32 / char_aspect) as usize, height)
    } else {
        (width, (width as f32 * char_aspect) as usize)
    };
    let width = match matches.value_of("width") {
        Some(w) => w.parse::<usize>().unwrap_or(width),
        None => width
    };
    let height = match matches.value_of("height") {
        Some(h) => h.parse::<usize>().unwrap_or(height),
        None => height
    };
    let rotation = match matches.value_of("up") {
        Some("x") => Rotation3::new(Vector3::new(0.0, 0.0, -PI / 2.0)),
        Some("y") => Rotation3::new(Vector3::new(0.0, 0.0, 0.0)),
        Some("z") => Rotation3::new(Vector3::new(-PI / 2.0, 0.0, 0.0)),
        _ => Rotation3::new(Vector3::new(-PI / 2.0, 0.0, 0.0))
    };
    let mode = match matches.value_of("mode") {
        Some("s") | Some("static") => Mode::Static,
        Some("r") | Some("rotation") => Mode::Rotation,
        _ => Mode::Static
    };

    let mut stlfile = File::open(Path::new(filename)).expect("error while opening stl file");
    let binfile = stl::read_stl(&mut stlfile).unwrap();

    let mut verts = Vec::new();
    let mut faces = Vec::new();
    for x in binfile.triangles {
        let n = verts.len();
        faces.push(sf::Patch::Tri(n, n+1, n+2));
        verts.push((x.v1, x.normal));
        verts.push((x.v2, x.normal));
        verts.push((x.v3, x.normal));
    }

    let mi = verts.iter().fold([f32::MAX; 3], |mut thing, val| {
        for i in 0..3 {
            if thing[i] > val.0[i] {
                thing[i] = val.0[i]
            }
        }
        thing
    });
    let ma = verts.iter().fold([f32::MIN; 3], |mut thing, val| {
        for i in 0..3 {
            if thing[i] < val.0[i] {
                thing[i] = val.0[i]
            }
        }
        thing
    });

    let c = 0.5*Vector3::new(ma[0]+mi[0], ma[1]+mi[1], ma[2]+mi[2]);
    let l = (ma[0]-mi[0]).max(ma[1]-mi[1]).max(ma[2]-mi[2]);

    let verts = verts.into_iter().map(|(p,n)| {
        (2.0*l.recip()*(Vector3::new(p[0], p[1], p[2])-c), Vector3::new(n[0], n[1], n[2]))
    }).map(|(p, n)| {
        (rotation.rotate(&p), rotation.rotate(&n))
    }).collect();

    let vertex = move |rot: &Rotation3<f32>, &(p, n): &(Vector3<f32>, Vector3<f32>)| {
        let rp = rot.rotate(&p);
        let rn = rot.rotate(&n);
        (Vector4::new(rp.x, rp.y, rp.z, 1.0), (rn.z+rn.x)/1.4)
    };

    let fragment = |_: &Rotation3<f32>, v: &f32| {
        Some(sf::Pixel::Grayscale(v.max(0.0)))
    };

    let mut buf = sf::Buffer::new(width, height, None);
    let mut rot = Rotation3::new(Vector3::new(0.0,0.0,0.0));
    match mode {
        Mode::Rotation => loop {
            rot.append_rotation_mut(&Vector3::new(0.0,0.05,0.0));
            buf.clear();
            sf::process(&mut buf, &rot, &verts, &faces, &vertex, &fragment);
            print_mat(&buf);
            print!("\x1B[{}A\x1B[1G", height - 1);
            stdout().flush().unwrap();
        },
        Mode::Static => {
            buf.clear();
            sf::process(&mut buf, &rot, &verts, &faces, &vertex, &fragment);
            print_mat(&buf);
        }
    }
}
