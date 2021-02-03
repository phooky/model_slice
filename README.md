# model_slice
A command-line tool for slicing STL models into two parts along the Z axis. Uses [`lyon`](https://docs.rs/lyon/0.17.5/lyon/) to tesselate the constructed faces.

## Build instructions
1. Install the rust toolchain, if you don't already have it.
2. `cargo build --release`

```
Simple solid model slicer 
Cuts a model along a plane orthogonal to the z-axis

USAGE:
    model_slice [OPTIONS] <FILE>

FLAGS:    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -b <bottom>          Output path for STL below slicing plane
    -e <edge>            Output path for SVG of midplane slice
    -t <top>             Output path for STL above slicing plane
    -z <z-height>        Cut the model at the given z-height [default: 0]
ARGS:
    <FILE>    
```
