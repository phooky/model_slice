// Sweep across edges to generate triangles of faces.
//
#![allow(dead_code)]
#![allow(unused_variables)]

use std::cmp::Ordering;
use stl_io;

#[derive(Copy,Clone,PartialEq,PartialOrd)]
struct Point { x : f32, y : f32 }

#[derive(Copy,Clone,PartialEq)]
pub struct Edge { a : Point, b : Point, slope : f32 }

impl PartialOrd for Edge { 
    // We want to sort by start position followed by angle.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let c1 = self.a.partial_cmp(&other.a);
        if c1 == Some(Ordering::Equal) {
            // We don't actually need to do the arcsin here! Ordering by
            // the slope is fine.
            self.slope.partial_cmp(&other.slope)
        } else { c1 }
    }
}

impl Edge {
    pub fn new(a : &stl_io::Vertex, b : &stl_io::Vertex) -> Edge {
        let a = Point { x : a[0], y : a[1] };
        let b = Point { x : b[0], y : b[1] };
        let (p1, p2) = if a <= b { (a, b) } else { (b, a) };
        Edge { a : p1, b : p2, slope : Edge::slope(p1,p2) }
    }

    fn slope(a : Point, b : Point) -> f32 {
        let dy = b.y - a.y;
        let dx = b.x - a.x;
        if dx == 0.0 {
            if dy > 0.0 { f32::INFINITY }
            else if dy < 0.0 { f32::NEG_INFINITY }
            else { 0.0 }
        } else {
            dy / dx
        }
    }

    fn reversed(&self) -> Edge {
        Edge { a : self.b, b : self.a, slope : -self.slope }
    }

    fn locate_pt(&self, pt : &Point) -> PtLoc {
        if !self.slope.is_infinite() {
            let y = self.a.y + (self.slope * (pt.x - self.a.x));
            if pt.y > y { PtLoc::Below }
            else if pt.y < y { PtLoc::Above }
            else { PtLoc::On }
        } else {
            if pt.y < self.a.y { PtLoc::Above }
            else if pt.y > self.b.y { PtLoc::Below }
            else { PtLoc:: On }
        }
    }
}

fn prepare_edgelist(edges : &mut Vec<Edge>) {
    // Sort by first point and angle
    edges.sort_by(|a, b| a.partial_cmp(b).unwrap());
}

struct MonoPoly {
    upper : Edge,
    lower : Edge,
}

enum PtLoc {
    Above,
    On,
    Below,
}

enum EdgeDisposition {
    Outside,
    Inside,
    Crossing,
    Extends([Point;3]),
}

enum SweepDisposition {
    Unchanged,
    Discard([Point;3]),
}

impl MonoPoly {
    fn handle_edge(&mut self, edge : &Edge) -> EdgeDisposition {
        // extension cases
        if edge.a == self.upper.b {
            let prior = self.upper;
            self.upper = *edge;
            EdgeDisposition::Extends([prior.a,prior.b,self.lower.a])
        } else if edge.a == self.lower.b {
            let prior = self.lower;
            self.lower = *edge;
            EdgeDisposition::Extends([prior.b,prior.a,self.upper.a])
        } else if edge.slope.is_infinite() && edge.b == self.upper.b {
            let prior = self.upper;
            self.upper = edge.reversed();
            EdgeDisposition::Extends([prior.a,prior.b,self.lower.a])
        } else if edge.slope.is_infinite() && edge.b == self.lower.b {
            let prior = self.lower;
            self.lower = edge.reversed();
            EdgeDisposition::Extends([prior.b,prior.a,self.upper.a])
        } else {
            let aloc = self.upper.locate_pt(&edge.a);
            let bloc = self.lower.locate_pt(&edge.a);
            use PtLoc::*;
            use EdgeDisposition::*;
            match (aloc, bloc) {
                (Above, Above) => Outside,
                (Below, Below) => Outside,
                (Below, Above) => Inside,
                _ => Crossing,
            }
        }
    }

    fn handle_sweep(& self, sweep : f32) -> SweepDisposition {
        use SweepDisposition::*;
        if sweep > self.upper.b.x || sweep > self.lower.b.x {
            if self.upper.b == self.lower.b {
                Discard([self.upper.a,self.upper.b,self.lower.b])
            } else {
                // Probably error out?
                Unchanged
            }
        } else { Unchanged }
    }

}

fn make_vert(p : &Point, z : f32) -> stl_io::Vertex {
    stl_io::Vertex::new( [p.x, p.y, z] )
}

fn make_tri(v : &[Point;3], z : f32) -> stl_io::Triangle {
    stl_io::Triangle { 
        normal : stl_io::Normal::new([0.0,0.0,-1.0]),
        vertices : [ 
            make_vert(&v[0],z),
            make_vert(&v[1],z), 
            make_vert(&v[2],z) ],
    }
}


// Rough outline:
// Start with empty mp list
// MP list constraints: always sorted on Y, no crossing over boundries of adjacent
//      MPs, always valid for current sweep position
// For each element of edge list:
//   Scan list for location.
//     If outside all: immediately retrieve next edge to find start of new MP.
//     If extends an existing MP: update MP and eject a triangle.
//     If closes an existing MP: MISSING CASE-- mark for removal, do not remove
//                               remove MP and eject a triangle.
//     If inside an existing MP: immediately retrieve next edge and split MP.
//     If CROSSES an existing MP: HANDLE EDGE CASE
pub
fn sweep_edges(mut edges: Vec<Edge>, z : f32) -> Vec<stl_io::Triangle> {
    let mut tri_list = Vec::new();
    let mut mp_list = Vec::<MonoPoly>::new();
    prepare_edgelist(&mut edges);
    let mut iter = edges.iter();
    while let Some(edge) = iter.next() {
        let sweep = edge.a.x;
        // Update MPs with the sweep line; some may request to be closed
        mp_list.retain(|mp| {
            match mp.handle_sweep(sweep) {
                SweepDisposition::Unchanged => true,
                SweepDisposition::Discard(tri) => {
                    tri_list.push(make_tri(&tri,z));
                    false
                }
            }
        });
        // Attempt to find a handler
        let mut found = false;
        for mp in &mut mp_list {
            use EdgeDisposition::*;
            match mp.handle_edge(edge) {
                Outside => {},
                Crossing => { panic!("Edge crossing detected!"); },
                Inside => { 
                },
                Extends(tri) => {
                    found = true;
                    tri_list.push(make_tri(&tri,z));
                },
            }
        }
        if !found {
            if let Some(edge2) = iter.next() {
                if edge.a == edge2.a {
                    mp_list.push(MonoPoly { upper : *edge, lower : *edge2 });
                } else if edge.a == edge2.b {
                    mp_list.push(MonoPoly { lower : *edge, upper : edge2.reversed() });
                } else {
                    panic!("Unmatched edge!");
                }
            } else {
                panic!("Dangling edge!");
            }
        }
    }
    // Close remaining MPs
    mp_list.retain(|mp| {
        match mp.handle_sweep(f32::INFINITY) {
                SweepDisposition::Unchanged => true,
                SweepDisposition::Discard(tri) => {
                    tri_list.push(make_tri(&tri,z));
                    false
                }
        }
    });
    tri_list
}
