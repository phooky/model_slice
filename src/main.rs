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
    // Sort vertices in order of ascending z-height.
    // The 'sense' boolean is true if the cycle of the
    // vertices has been reversed.
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

fn correct_sense(tri : &mut Triangle, sense : bool) {
    // Correct the sense of the triangle
    if sense {
        let tmp = tri.vertices[1];
        tri.vertices[2] = tri.vertices[1];
        tri.vertices[1] = tmp;
    }
}

struct SplitModel {
    zplus : Vec<Triangle>,
    zminus : Vec<Triangle>,
    edge : Vec<Segment>,
}

impl SplitModel {
    fn new() -> SplitModel {
        SplitModel{ zplus : Vec::new(), zminus : Vec::new(), edge : Vec::new() }
    }

    fn add_tri(&mut self, mut t2 : Triangle, sense : bool, zplus : bool) {
        correct_sense(&mut t2, sense);
        if zplus { 
            self.zplus.push(t2); 
        } else {
            self.zminus.push(t2);
        }
    }

    fn split_tri(&mut self, tri : &Triangle, z : f32) {
        let original = tri;
        let (tri, sense) = reorder(&original);
        let [v0, v1, v2] = tri.vertices;
        if v2[2] <= z { // case A/B/C
            self.zminus.push(original.clone());
            if v1[2] == z { // case C
                self.edge.push(Segment { vertices : [v1,v2] });
            }
        } else if v1[2] < z { // case D 
            let x = intersect_plane(&v1,&v2,z);
            let y = intersect_plane(&v0,&v2,z);
            self.add_tri( Triangle { normal : tri.normal,
                vertices : [v0,v1,x] }, sense, false );
            self.add_tri( Triangle { normal : tri.normal,
                vertices : [v0,x,y] }, sense, false );
            self.add_tri( Triangle { normal : tri.normal,
                vertices : [x,v2,y] }, sense, true );
            self.edge.push(Segment { vertices : [x,y] });
        } else if v1[2] == z { // case E 
            let x = intersect_plane(&v0,&v2,z);
            self.add_tri( Triangle { normal : tri.normal,
                vertices : [v0,v1,x] }, sense, false );
            self.add_tri( Triangle { normal : tri.normal,
                vertices : [v1,v2,x] }, sense, true );
            self.edge.push(Segment { vertices : [x,v1] });
        } else if v0[2] < z { // case F 
            let x = intersect_plane(&v0,&v1,z);
            let y = intersect_plane(&v0,&v2,z);
            self.add_tri( Triangle { normal : tri.normal,
                vertices : [v0,x,y] }, sense, false );
            self.add_tri( Triangle { normal : tri.normal,
                vertices : [y,x,v2] }, sense, true );
            self.add_tri( Triangle { normal : tri.normal,
                vertices : [x,v1,v2] }, sense, true );
            self.edge.push(Segment { vertices : [x,y] });
        } else if v0[2] >= z {
            self.zminus.push(original.clone());
            if v1[2] == z { // case G
                self.edge.push(Segment { vertices : [v0,v1] });
            }
        }
    }
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

fn intersect_plane(a : &Vertex, b : &Vertex, z : f32) -> Vertex {
    let prop = (z - a[2])/(b[2] - a[2]);
    let x = a[0] + prop*(b[0]-a[0]);
    let y = a[1] + prop*(b[1]-a[1]);
    Vertex::new([x,y,z])
}

fn main() {
    let clap = App::new("Simple solid model slicer")
        .about("Cuts a model along a plane orthogonal to the z-axis")
        .arg(Arg::with_name("z-height")
             .short("z")
             .help("Cut the model at the given z-height")
             .default_value("0")
             .takes_value(true))
        .arg(Arg::with_name("top")
             .short("t")
             .help("Output path for STL above slicing plane")
             .takes_value(true))
        .arg(Arg::with_name("bottom")
             .short("b")
             .help("Output path for STL below slicing plane")
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
                        let x = intersect_plane(&tri.vertices[1],&tri.vertices[2],z);
                        let y = intersect_plane(&tri.vertices[0],&tri.vertices[2],z);
                        below.push( Triangle { normal : tri.normal,
                            vertices : [tri.vertices[0],tri.vertices[1],x] } );
                        below.push( Triangle { normal : tri.normal,
                            vertices : [tri.vertices[0],x,y] } );
                        above.push( Triangle { normal : tri.normal,
                            vertices : [x,y,tri.vertices[2]] } );
                        on.push(Segment { vertices : [x,y] });
                    } else if tri.vertices[1][2] == z { // case E 
                        let x = intersect_plane(&tri.vertices[0],&tri.vertices[2],z);
                        below.push( Triangle { normal : tri.normal,
                            vertices : [tri.vertices[0],x,tri.vertices[1]] } );
                        above.push( Triangle { normal : tri.normal,
                            vertices : [tri.vertices[1],x,tri.vertices[2]] } );
                        on.push(Segment { vertices : [x,tri.vertices[1]] });
                    } else if tri.vertices[0][2] < z { // case F 
                        let x = intersect_plane(&tri.vertices[0],&tri.vertices[1],z);
                        let y = intersect_plane(&tri.vertices[0],&tri.vertices[2],z);
                        above.push( Triangle { normal : tri.normal,
                            vertices : [tri.vertices[2],tri.vertices[1],x] } );
                        above.push( Triangle { normal : tri.normal,
                            vertices : [tri.vertices[2],x,y] } );
                        below.push( Triangle { normal : tri.normal,
                            vertices : [y,x,tri.vertices[0]] } );
                        on.push(Segment { vertices : [x,y] });
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
    match matches.value_of("bottom") {
        Some(path) => {
            let mut f = File::create(path).unwrap();
            stl_io::write_stl(&mut f,below.iter());
        },
        None => {},
    }
    match matches.value_of("top") {
        Some(path) => {
            let mut f = File::create(path).unwrap();
            stl_io::write_stl(&mut f,above.iter());
        },
        None => {},
    }
    println!("Triangle count {} above, {} below, {} segments",above.len(),below.len(), on.len());
    println!("Slicing model at z-height {}",z);
}
