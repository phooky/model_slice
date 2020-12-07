extern crate clap;
use clap::{Arg, App};
use std::fs::File;

fn main() {
    let clap = App::new("Simple solid model slicer")
        .about("Cuts a model along a plane orthogonal to the z-axis")
        .arg(Arg::with_name("z-height")
             .short("z")
             .help("Cut the model at the given z-height")
             .default_value("0")
             .takes_value(true))
        .arg(Arg::with_name("FILE")
             .required(true)
             .index(1));
    let matches = clap.get_matches();

    let z : f32 = matches.value_of("z-height").unwrap()
        .parse().unwrap();
    let path = matches.value_of("FILE").unwrap();
    let f = File::open(path).unwrap();

    println!("Loading model {}",path);
    println!("Slicing model at z-height {}",z);
}
