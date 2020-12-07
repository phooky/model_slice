extern crate clap;
use clap::{Arg, App};
use std::fs::File;

enum RelPos {
    ABOVE,
    BELOW,
    ON,
}

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
    let mut f = File::open(path).unwrap();

    println!("Loading model {}",path);
    let stl = stl_io::create_stl_reader(&mut f).unwrap();
    let mut tcount = 0;
    for tri in stl {
        tcount = tcount + 1;
    }
    println!("Triangle count {}",tcount);
    println!("Slicing model at z-height {}",z);
}
