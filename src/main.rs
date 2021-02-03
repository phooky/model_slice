extern crate clap;
extern crate stl_io;
extern crate float_cmp;
extern crate svg;

use float_cmp::ApproxEq;
use clap::{Arg, App};
use std::fs::File;
use stl_io::{Vertex,Triangle};

mod build_face;
use build_face::{Segment,build_loops,build_faces};

mod sweep_e2f;
use sweep_e2f::{Edge,sweep_edges};

fn build_edge(a : &Vertex, b : &Vertex) -> Option<Edge> {
    if a[0].approx_eq(b[0], (0.0,2)) && a[1].approx_eq(b[1], (0.0,2)) {
        None
    } else {
        Some(Edge::new(&a, &b))
    }
}

fn build_seg(x : &Vertex, y : &Vertex) -> Option<Segment> {
    if x[0].approx_eq(y[0], (0.0,2)) && x[1].approx_eq(y[1], (0.0,2)) {
        None
    } else {
        Some(Segment::new(x, y))
    }
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
        let tmp = tri.vertices[2];
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
                self.edge.push(Segment::new(&v1,&v2));
            }
        } else if v1[2] < z { // case D 
            let x = intersect_plane(&v1,&v2,z);
            let y = intersect_plane(&v0,&v2,z);
            // Robustness check: don't create zero-size slivers
            if let Some(segment) = build_seg(&x,&y) {
                self.add_tri( Triangle { normal : tri.normal,
                    vertices : [v0,v1,x] }, sense, false );
                self.add_tri( Triangle { normal : tri.normal,
                    vertices : [v0,x,y] }, sense, false );
                self.add_tri( Triangle { normal : tri.normal,
                    vertices : [x,v2,y] }, sense, true );
                self.edge.push(segment);
            } else {
                self.zminus.push(original.clone());
            }
        } else if v1[2] == z { // case E 
            // RBST: x == v1? That would be a zero-size sliver already...
            let x = intersect_plane(&v0,&v2,z);
            self.add_tri( Triangle { normal : tri.normal,
                vertices : [v0,v1,x] }, sense, false );
            self.add_tri( Triangle { normal : tri.normal,
                vertices : [v1,v2,x] }, sense, true );
            self.edge.push(Segment::new(&x,&v1));
        } else if v0[2] < z { // case F 
            let x = intersect_plane(&v0,&v1,z);
            let y = intersect_plane(&v0,&v2,z);
            // Robustness check: don't create zero-size slivers
            if let Some(segment) = build_seg(&x,&y) {
                self.add_tri( Triangle { normal : tri.normal,
                    vertices : [v0,x,y] }, sense, false );
                self.add_tri( Triangle { normal : tri.normal,
                    vertices : [y,x,v2] }, sense, true );
                self.add_tri( Triangle { normal : tri.normal,
                    vertices : [x,v1,v2] }, sense, true );
                self.edge.push(Segment::new(&x,&y));
            } else {
                self.zplus.push(original.clone());
            }
        } else if v0[2] >= z {
            self.zplus.push(original.clone());
            if v1[2] == z { // case G
                self.edge.push(Segment::new(&v0,&v1));
            }
        } else { println!("CASE X"); }
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
        .arg(Arg::with_name("edge")
             .short("e")
             .help("Output path for SVG of midplane slice")
             .takes_value(true))
        .arg(Arg::with_name("FILE")
             .required(true)
             .index(1));
    let matches = clap.get_matches();

    let z : f32 = matches.value_of("z-height").unwrap()
        .parse().unwrap();
    let path = matches.value_of("FILE").unwrap();
    let mut f = File::open(path).unwrap();
    let mut sm = SplitModel::new();

    println!("Loading model {}",path);
    let stl = stl_io::create_stl_reader(&mut f).unwrap();
    for tri_res in stl {
        match tri_res {
            Ok(tri) => sm.split_tri(&tri, z),
            _ => {},
        }
    }
    println!("Triangle count {} above, {} below, {} segments",sm.zplus.len(),sm.zminus.len(), sm.edge.len());
    println!("Slicing model at z-height {}",z);
    let loops = build_loops(&sm.edge);
    let mut cut_face = build_faces(&loops,z);
    match matches.value_of("top") {
        Some(path) => {
            let mut f = File::create(path).unwrap();
            sm.zplus.append(&mut cut_face.clone());
            stl_io::write_stl(&mut f,sm.zplus.iter()).unwrap();
        },
        None => {},
    }
    match matches.value_of("bottom") {
        Some(path) => {
            let mut f = File::create(path).unwrap();
            // Invert the normals on the cut face
            let mut cf = cut_face.clone();
            let normal = stl_io::Vector::new([0.0,0.0,-1.0]);
            for tri in cf.iter_mut() {
                correct_sense(tri, true);
                tri.normal = normal;
            }
            sm.zminus.append(&mut cf);
            stl_io::write_stl(&mut f,sm.zminus.iter()).unwrap();
        },
        None => {},
    }
    let scale = 5.0;
    println!("Loop count: {}", loops.len());
    match matches.value_of("edge") {
        Some(path) => {
            let mut document = svg::Document::new()
                .set("viewBox", (0, 0, 800, 800));
            for l in loops {
                let mut data = svg::node::element::path::Data::new();
                data = data.move_to((l.pts[0][0]*scale+400.0,l.pts[0][1]*scale+400.0));
                for pt in l.pts.iter().skip(1) {
                    data = data.line_to((pt[0]*scale+400.0,pt[1]*scale+400.0));
                }
                if l.closed {
                    data = data.line_to((l.pts[0][0]*scale+400.0,l.pts[0][1]*scale+400.0));
                }
                let path = svg::node::element::Path::new()
                    .set("fill", "none")
                    .set("stroke", "black")
                    .set("stroke-width", 0.5)
                    .set("d", data);

                document = document.add(path);
            }
            svg::save(path, &document).unwrap();
        },
        None => {},
    }

}
