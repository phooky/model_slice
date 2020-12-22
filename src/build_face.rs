use std::hash::{Hash, Hasher};
use stl_io::Vertex;

// I much prefer x/y to array indexing
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
