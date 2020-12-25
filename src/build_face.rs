use kd_tree::{KdPoint, KdTree};
use stl_io::Vertex;
extern crate typenum;

// I much prefer x/y to array indexing
struct Vertex2 { x : f32, y : f32 }

// A segment is a _directed_ pair of 2d vertices
pub struct Segment { a : Vertex2, b: Vertex2 }

impl Segment {
    pub fn new(v1 : &Vertex, v2 : &Vertex) -> Segment {
        Segment { 
            a : Vertex2 { x: v1[0], y : v1[1] }, 
            b : Vertex2 { x: v2[0], y : v2[1] }, }
    }
}

pub struct Loop {
    pts : Vec<Vertex2>,
    closed : bool,
}

struct TreeItem {
    v : Vertex2,
    idx : usize,
}

impl KdPoint for TreeItem {
    type Scalar = f32;
    type Dim = typenum::U2; 
    fn at(&self, k: usize) -> f32 { if k == 0 { self.v.x } else {self.v.y} }
}

pub fn build_loops(segs : &Vec<Segment>) -> Vec<Loop> {
    let mut loops = Vec::new();
    loops
}

// Alternate Strategy:
// throw every segment in a K-D tree twice. Key is vertex. Value
// is a list of vertices or chains. (Should everything start as a
// 1-length chain? It seems clever, but may involve more data 
// reshuffling than we like.)
//
// If we start out trying to build the chains as we insert, we may
// run into issues with intersecting chains, and a lot of chain
// merging. Intersecting chains we can deal with by keeping pointers
// to chains (although position annotations would suck), but then we've
// got a separate split chain issue, etc. Doing all insertions first and
// keeping track of X points separately may do the trick.

