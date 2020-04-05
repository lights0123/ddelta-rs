use std::fs::File;
use std::io::{Read, BufWriter};

use ddelta::{apply, generate};
use xz2::write::XzEncoder;
use xz2::read::XzDecoder;
fn main() {
    let now = std::time::Instant::now();
    {
        let mut old = vec![];
        let mut new = vec![];
        File::open("old").unwrap().read_to_end(&mut old).unwrap();
        File::open("new").unwrap().read_to_end(&mut new).unwrap();
        let mut patch = File::create("patch").unwrap();
        generate(&old, &new, &mut BufWriter::new(patch)).unwrap();
    }
    // {
    //     let mut old = File::open("old").unwrap();
    //     let mut patch = File::open("patch").unwrap();
    //     let mut new = File::create("new2").unwrap();
    //     let mut decoder = XzDecoder::new(patch);
    //     apply(&mut decoder, &mut old, &mut new).unwrap();
    // }
    println!("{:?}", now.elapsed());
}
