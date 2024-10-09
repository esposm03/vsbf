use std::env::args;

use vsbf::{FileHeader, Vsbf};

fn main() {
    let buf = std::fs::read(args().nth(1).unwrap()).unwrap();
    let (_data, file) = Vsbf::parse(&buf).unwrap();

    let (_, hdr) = FileHeader::parse(&buf).expect("Failed to parse header");
    println!("File header: {hdr:?}");

    println!("\nSegments:");
    for segm in file.segments() {
        println!("- {segm:?}");
    }

    println!("\nSections:");
    for sect in file.sections() {
        println!("- {sect:x?}");
    }
}
