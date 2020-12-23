use std::hash::{Hash, Hasher};
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;

use stl_io::Vertex;

// I much prefer x/y to array indexing
#[derive(PartialEq, Eq)]
struct Vertex2 { x : f32, y : f32 }

// A segment is a _directed_ pair of 2d vertices
pub struct Segment { a : Vertex2, b: Vertex2 }

impl Hash for Vertex2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
    }
}

impl Segment {
    pub fn new(v1 : &Vertex, v2 : &Vertex) -> Segment {
        Segment { 
            a : Vertex2 { x: v1[0], y : v1[1] }, 
            b : Vertex2 { x: v2[0], y : v2[1] }, }
    }
}

/*
// We're going to attempt the hash table approach for now.
pub fn build_loops(segs : &Vec<Segment>) -> Vec<Vec<Vertex2> > {
    let mut loops = Vec::new();
    let mut hashtab = HashMap::new();
    for seg in segs {
        if !hashtab.contains_key(seg.a) {
            hashtab.insert(seg.a,Vec::new());
        }
        if !hashtab.contains_key(seg.b) {
            hashtab.insert(seg.b,Vec::new());
        }
        hashtab.get_mut(seg.b).unwrap().push(seg.a);
        hashtab.get_mut(seg.a).unwrap().push(seg.b);
    }
    loops
}*/

// NOPEING OUT on hashing floats. This is a bullshit idea.

