extern crate clap;
extern crate stl_io;
use clap::{Arg, App};
use std::fs::File;
use stl_io::{Vertex,Triangle};

enum RelPos {
    ABOVE,
    BELOW,
    ON,
}

struct Segment {
    vertices : [Vertex; 2],
}

fn reorder(tri : &Triangle) -> (Triangle, bool) {
    // Reorder vertices in the triangle and add a bit of
    // information to indicate if the sense of the triangle
    // has changed.
    // if z1 < z0 swap 0 1
    // if z2 < z0 swap 0 2
    // if z1 < z2 swap 1 2
    let mut rv = tri.clone();
    let mut sense = false;
    let mut cond_swap = |a : usize, b : usize| {
        if rv.vertices[a][2] > rv.vertices[b][2] {
            let tmp = rv.vertices[a]; 
            rv.vertices[a] = rv.vertices[b];
            rv.vertices[b] = tmp;
            sense = !sense;
        }
    };
    cond_swap(0,1);
    cond_swap(0,2);
    cond_swap(1,2);
    (rv, sense)
}

fn check(tri : &Triangle, z : f32) -> RelPos {
    let mut above = 0;
    let mut below = 0;
    for i in 0..3 {
        let tz = tri.vertices[i][2];
        if tz > z { above = above + 1; }
        if tz < z { below = below + 1; }
    }
    if above == 3 {
        RelPos::ABOVE
    } else if below == 3 {
        RelPos::BELOW
    } else {
        RelPos::ON
    }
}

fn main() {
    let clap = App::new("Simple solid model slicer")
        .about("Cuts a model along a plane orthogonal to the z-axis")
        .arg(Arg::with_name("z-height")
             .short("z")
             .help("Cut the model at the given z-height")
             .default_value("0")
             .takes_value(true))
        .arg(Arg::with_name("FILE")
             .required(true)
             .index(1));
    let matches = clap.get_matches();

    let z : f32 = matches.value_of("z-height").unwrap()
        .parse().unwrap();
    let path = matches.value_of("FILE").unwrap();
    let mut f = File::open(path).unwrap();

    println!("Loading model {}",path);
    let stl = stl_io::create_stl_reader(&mut f).unwrap();
    let mut above = Vec::<Triangle>::new();
    let mut below = Vec::<Triangle>::new();
    let mut on = Vec::<Segment>::new();
    for tri_res in stl {
        match tri_res {
            Ok(tri) => match check(&tri, z) {
                RelPos::ABOVE => { above.push(tri.clone()); } // case A
                RelPos::BELOW => { below.push(tri.clone()); } // case I
                RelPos::ON => {
                    let (tri, sense) = reorder(&tri);
                    if tri.vertices[2][2] == z {
                        if tri.vertices[1][2] == z { // case C
                            above.push(tri.clone());
                            on.push(Segment { vertices : [tri.vertices[1],tri.vertices[2]] });
                        } else { // case B
                            above.push(tri.clone());
                        }
                    } else if tri.vertices[1][2] < z { // case D
                    } else if tri.vertices[1][2] == z { // case E
                    } else if tri.vertices[0][2] < z { // case F
                    } else if tri.vertices[0][2] == z {
                        if tri.vertices[1][2] == z { // case G
                            below.push(tri.clone());
                            on.push(Segment { vertices : [tri.vertices[0],tri.vertices[1]] });
                        } else { // case H
                            below.push(tri.clone());
                        }
                    }
                    //println!("Z: {} {} {} {:?}", tri.vertices[0][2], tri.vertices[1][2], 
                    //         tri.vertices[2][2], sense);
                }
            },
            _ => {}
        }
    }
    println!("Triangle count {} above, {} below, {} segments",above.len(),below.len(), on.len());
    println!("Slicing model at z-height {}",z);
}
