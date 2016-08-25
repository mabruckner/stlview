extern crate starfield_render;
extern crate clap;
extern crate stl;
extern crate nalgebra;

use std::fs::File;
use std::path::Path;
use starfield_render as sf;
use clap::{App, Arg};
use nalgebra::{Vector3, Vector4, Rotation3, Rotation, Rotate};
use std::f32;

fn print_mat(buf: &sf::Buffer<sf::Pixel>)
{
    for y in 0..buf.height {
        for x in 0..buf.width {
            match buf.get((x,y)) {
                &Some((ref col, _)) => print!("\x1B[48;5;{}m ", sf::to_256_color(col, x, y)),
                &None => print!("\x1B[48;5;0m ")
            }
        }
        println!("");
    }
}



fn main() {
    let matches = App::new("stlview")
                        .about("Displays stl files to the terminal.")
                        .arg(Arg::with_name("file")
                             .required(true))
                        .arg(Arg::with_name("width")
                             .short("w")
                             .long("width"))
                        .arg(Arg::with_name("height")
                             .short("h")
                             .long("height"))
                        .get_matches();
    let filename = matches.value_of("file").unwrap();
    let (width, height) = (100, 50);

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
    }).collect();

    let vertex = move |rot: &Rotation3<f32>, &(p, n): &(Vector3<f32>, Vector3<f32>)| {
        let rp = rot.rotate(&p);
        let rn = rot.rotate(&n);
        (Vector4::new(rp.x, rp.y, rp.z, 1.0), (rn.z+rn.x)/1.4)
    };

    let fragment = |u: &Rotation3<f32>, v: &f32| {
        Some(sf::Pixel::Grayscale(v.max(0.0)))
    };

    let mut buf = sf::Buffer::new(width, height);
    let mut rot = Rotation3::new(Vector3::new(0.0,0.0,0.0));
    loop {
        rot.append_rotation_mut(&Vector3::new(0.0,0.05,0.0));
        buf.clear();
        sf::process(&mut buf, &rot, &verts, &faces, &vertex, &fragment);
        print_mat(&buf);
        println!("\x1B[{}A", height+1);
    }
}
