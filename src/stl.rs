use nom;
use nom::{float, le_f32, le_u32};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

pub type Vertex = [f32; 3];

pub type Triangle = [Vertex; 3];

pub type Solid = Vec<Triangle>;


named!(vert_bin<&[u8], Vertex>, map!(tuple!(le_f32, le_f32, le_f32), |(a,b,c)| { [a,b,c] }));

named!(tri_bin<&[u8], Triangle>, map!(do_parse!(take!(12) >> t:tuple!(vert_bin, vert_bin, vert_bin) >> take!(2) >> (t)), |(a,b,c)| { [a,b,c] }));

named!(solid_bin<&[u8], Solid>, do_parse!(take!(80) >> num: le_u32 >> faces: count!(tri_bin, num as usize) >> (faces)));

named!(vert_text<&[u8], Vertex>, map!(ws!(preceded!(tag!("vertex"), tuple!(float, float, float))), |x:(f32, f32, f32)| { [x.0, x.1, x.2] }));

named!(tri_text<&[u8], Triangle>, map!(ws!(do_parse!(tag!("facet") >> tag!("normal") >> float >> float >> float >> tag!("outer") >> tag!("loop") >> a:vert_text >> b:vert_text >> c:vert_text >> tag!("endloop") >> tag!("endfacet") >> (a, b, c))), |(a,b,c)|{ [a,b,c]}));

named!(solid_text<&[u8], Solid>, alt!(ws!(do_parse!(tag!("solid") >> tris: many0!(tri_text) >> tag!("endsolid") >> (tris))) | ws!(do_parse!(tag!("solid") >> name: is_not!("\r\n") >> tris: many0!(tri_text) >> tag!("endsolid") >> tag!(name) >> (tris)))));

pub fn from_ascii(text: &[u8]) -> Option<Solid> {
    match solid_text(text) {
        nom::IResult::Done(_, o) => Some(o),
        x => {
            println!("{:?}", x);
            None
        }
    }
}

pub fn from_bin(bin: &[u8]) -> Option<Solid> {
    match solid_bin(bin) {
        nom::IResult::Done(_, o) => Some(o),
        _ => None
    }
}

pub fn read_stl(data: &[u8]) -> Option<Solid> {
    match from_ascii(data) {
        Some(thing) => Some(thing),
        None => from_bin(data)
    }
}

pub fn compute_normal(face: &[[f32; 3]; 3]) -> [f32; 3] {
    let mut out = [0.0; 3];
    let mut l = 0.0;
    for i in 0..3 {
        let a = (i+1)%3;
        let b = (i+2)%3;
        out[i] = (face[1][a]-face[0][a])*(face[2][b]-face[0][b]) - (face[1][b]-face[0][b])*(face[2][a]-face[0][a]);
        l = l+out[i]*out[i];
    }
    l = l.sqrt();
    for i in 0..3 {
        out[i] = out[i]/l;
    }
    out
}
