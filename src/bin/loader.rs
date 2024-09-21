use core::slice;
use std::{env::args, ptr::null_mut};

use vsbf::Vsbf;

fn main() {
    let buf = std::fs::read(args().nth(1).unwrap()).unwrap();
    let (_data, _file) = Vsbf::parse(&buf).unwrap();

    let map = unsafe {
        let map = libc::mmap(null_mut(), 0x1000, 7, 0, 0, 0).cast::<u8>();
        slice::from_raw_parts_mut(map, 0x1000)
    };

    println!("data: {:x?}", map);
}
