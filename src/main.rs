use std::{env::args, fs::File, process::exit};

use vsbf::{PermissionFlags, SegmentHeader, Vsbf};

fn main() {
    if args().len() < 2 {
        eprintln!("Usage: {} <filename> [filename...]", args().next().unwrap());
        exit(1);
    }

    let filename = args().nth(1).unwrap();
    let buf = std::fs::read(&filename).unwrap();
    let (_, mut obj) = Vsbf::parse(&buf).unwrap();

    let sections = obj.sections();
    let start = sections[0].offset as usize;

    obj.add_segment(SegmentHeader {
        typ: 0,
        flags: PermissionFlags::all(),
        align: 0x1000,
        file: start as u32,
        mem: 0,
        file_size: sections[0].file_size as _,
        mem_size: sections[0].file_size as _,
    });

    let mut file = File::create(filename.trim_end_matches(".o")).unwrap();
    obj.write(&mut file).unwrap();
}
