use stl_io::{Vertex,Triangle};
use rstar::RTree;
use rstar::primitives::PointWithData;

// A segment is a _directed_ pair of 2d vertices
pub struct Segment { a : [f32;2], b: [f32;2] }

impl Segment {
    pub fn new(v1 : &Vertex, v2 : &Vertex) -> Segment {
        if v1[0] == v2[0] && v1[1] == v2[1] { panic!("empty segment"); }
        Segment { 
            a : [ v1[0], v1[1] ],
            b : [ v2[0], v2[1] ], 
        }
    }
    pub fn other(&self, from : &[f32;2]) -> [f32;2] {
        if self.a[0] == from[0] && self.a[1] == from[1] { self.b } else { self.a }
    }
}

pub struct Loop {
    pub pts : Vec<[f32;2]>,
    pub closed : bool,
}

type PointWithIndex = PointWithData<usize,[f32;2]>;

pub fn build_loops(segs : &Vec<Segment>) -> Vec<Loop> {
    let mut loops = Vec::new();
    let mut tree = RTree::new();
    for (idx, seg) in segs.iter().enumerate() {
        tree.insert(PointWithIndex::new(idx,seg.a));
        tree.insert(PointWithIndex::new(idx,seg.b));
    }
    while tree.size() > 0 {
        println!("--- BUILDING LOOP ---");
        println!("Seg list {}, tree size {}",segs.len(),tree.size());
        let mut l = Loop { pts: Vec::new(), closed : false };
        let mut point = tree.pop_nearest_neighbor(&[0.0,0.0]).unwrap();
        loop {
            let idx = point.data;
            let pos = point.position();
            l.pts.push(*pos);
            let nextpos = segs[idx].other(pos);
            //println!("From {:?} to {:?} via {}",pos,nextpos,idx);
            tree.remove(&PointWithIndex::new(idx,nextpos));
            let mut np = None;
            for candidate in tree.locate_all_at_point(&nextpos) { 
                np = match np {
                    None => Some(candidate),
                    Some(_) => {
                        for (idx, seg) in segs.iter().enumerate() {
                            println!("{} {:?} {:?}",idx,seg.a,seg.b);
                        }
                        panic!("X-point!")},
                }
            }
            if np.is_none() { 
                l.pts.push(nextpos);
                break; 
            }
            point = *np.unwrap();
            tree.remove(&point);
        }
        let first = l.pts[0];
        let last = l.pts[l.pts.len()-1];
        println!("Loop from {:?} to {:?}",first,last);
        if first[0] == last[0] && first[1] == last[1] {
            println!("CLOSED");
            l.closed = true;
        }
        loops.push(l);
        //break;
    }
    loops
}

use lyon::path::Path;
use lyon::tessellation::*;

fn lpoint(p : &[f32;2]) -> lyon::math::Point {
    use lyon::math::point;
    point( p[0], p[1] )
}

struct WithZ(f32);

impl FillVertexConstructor<Vertex> for WithZ {
    fn new_vertex(&mut self, vertex: FillVertex) -> Vertex {
        let pos = vertex.position().to_array();
        Vertex::new([pos[0],pos[1],self.0])
    }
}

pub fn build_faces(loops : &Vec<Loop>, z : f32) -> Vec<Triangle> {
    println!("*** BFACES ***");
    let mut builder = Path::builder();
    for l in loops {
        builder.begin(lpoint(&l.pts[0]));
        for p in l.pts.iter().skip(1) {
            builder.line_to(lpoint(p));
        }
        builder.end(l.closed);
    }
    let path = builder.build();
    let mut tess = FillTessellator::new();
    let mut buffers: VertexBuffers<Vertex, u16> = VertexBuffers::new();
    let result = tess.tessellate_path(
        &path,
        &FillOptions::default(),
        &mut geometry_builder::BuffersBuilder::new(&mut buffers,WithZ(z))
    );
    assert!(result.is_ok());
    let ilen = buffers.indices.len();
    println!("Triangle count: {} {}",ilen, ilen/3);
    let mut rv = Vec::new();
    let normal = stl_io::Vector::new([0.0,0.0,1.0]);
    for chunk in buffers.indices.chunks_exact(3) {
        let vs : Vec<Vertex> = chunk.iter().map(|idx| buffers.vertices[*idx as usize]).collect();
        rv.push( Triangle { vertices : [ vs[0], vs[1], vs[2] ],
            normal : normal } );
    }
    println!("{} triangles",rv.len());
    rv
}

