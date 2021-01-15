// Sweep across edges to generate triangles of faces.
//
#![allow(dead_code)]
#![allow(unused_variables)]

use std::cmp::Ordering;

#[derive(Copy,Clone,PartialEq,PartialOrd)]
struct Point { x : f32, y : f32 }


#[derive(Copy,Clone,PartialEq)]
struct Edge { a : Point, b : Point }

impl PartialOrd for Edge { 
    // We want to sort by start position followed by angle.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let c1 = self.a.partial_cmp(&other.a);
        if c1 == Some(Ordering::Equal) {
            // We don't actually need to do the arcsin here! Ordering by
            // the slope is fine.
            self.slope().partial_cmp(&other.slope())
        } else { c1 }
    }
}

impl Edge {
    fn new(a : &Point, b : &Point) -> Edge {
        if a <= b { Edge { a:*a, b:*b } } else { Edge { a:*b, b:*a } }
    }
    fn slope(&self) -> f32 {
        let dy = self.b.y - self.a.y;
        let dx = self.b.x - self.a.x;
        if dx == 0.0 {
            if dy > 0.0 { f32::INFINITY }
            else if dy < 0.0 { f32::NEG_INFINITY }
            else { 0.0 }
        } else {
            dy / dx
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
    Outside,
    Upper,
    Lower,
    Inside,
}

impl MonoPoly {
    fn locate_pt(pt : &Point) -> PtLoc {
        PtLoc::Outside
    }
}

