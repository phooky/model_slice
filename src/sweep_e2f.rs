// Sweep across edges to generate triangles of faces.
//
#![allow(dead_code)]
#![allow(unused_variables)]

use std::cmp::Ordering;
use stl_io::{Triangle, Vertex};

#[derive(Copy,Clone,PartialEq,PartialOrd)]
struct Point { x : f32, y : f32 }


#[derive(Copy,Clone,PartialEq)]
struct Edge { a : Point, b : Point, slope : f32 }

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
    fn new(a : &Point, b : &Point) -> Edge {
        let (p1, p2) = if a <= b { (*a, *b) } else { (*b, *a) };
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

impl MonoPoly {
    fn locate_pt(&self, pt : &Point) -> PtLoc {
            PtLoc::On
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
//     If closes an existing MP: remove MP and eject a triangle.
//     If inside an existing MP: immediately retrieve next edge and split MP.
//     If CROSSES an existing MP: HANDLE EDGE CASE

fn sweep_edges(mut edges: Vec<Edge>, z : f32) -> Vec<Triangle> {
    let mut tris = Vec::new();
    tris
}
