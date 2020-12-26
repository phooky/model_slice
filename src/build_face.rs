use stl_io::Vertex;
use rstar::RTree;
use rstar::primitives::PointWithData;

// A segment is a _directed_ pair of 2d vertices
pub struct Segment { a : [f32;2], b: [f32;2] }

impl Segment {
    pub fn new(v1 : &Vertex, v2 : &Vertex) -> Segment {
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
    pts : Vec<[f32;2]>,
    closed : bool,
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
        println!("--- BUILD LOOP ---");
        println!("Seg list {}, tree size {}",segs.len(),tree.size());
        let mut l = Loop { pts: Vec::new(), closed : false };
        let mut point = tree.pop_nearest_neighbor(&[0.0,0.0]).unwrap();
        loop {
            let idx = point.data;
            let pos = point.position();
            l.pts.push(*pos);
            let nextpos = segs[idx].other(pos);
            println!("From {:?} to {:?} via {}",pos,nextpos,idx);
            //tree.locate_all_at_point
            break;
        }
        loops.push(l);
        break;
    }
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

