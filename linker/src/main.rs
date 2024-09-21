use std::{env::args, fs::File, io::Write, process::exit};

use bytemuck::bytes_of;
use vsbf::{parse_file, PermissionFlags, SegmentHeader};

fn main() {
    if args().len() < 2 {
        eprintln!("Usage: {} <filename> [filename...]", args().next().unwrap());
        exit(1);
    }

    let filename = args().nth(1).unwrap();
    let buf = std::fs::read(&filename).unwrap();
    let (data, file) = parse_file(&buf).unwrap();

    let mut hdr = file.header;
    let mut segments = file.segments;
    let sections = file.sections;

    assert_eq!(hdr.num_segments, 0);
    assert_eq!(hdr.num_segments as usize, segments.len());
    assert_eq!(hdr.num_sections as usize, sections.len());

    let start = sections[0].offset as usize;
    let end = start + sections[0].file_size as usize;
    println!("{:x?}", &buf[start..end]);

    hdr.num_segments += 1;
    segments.push(SegmentHeader {
        typ: 0,
        flags: PermissionFlags::all(),
        align: 0x1000,
        file: start as u32,
        mem: 0,
        file_size: sections[0].file_size as _,
        mem_size: sections[0].file_size as _,
    });

    let mut file = File::create(filename.trim_end_matches(".o")).unwrap();
    file.write_all(b"VSBF").unwrap();
    file.write_all(bytes_of(&hdr)).unwrap();
    for seg in segments {
        file.write_all(bytes_of(&seg)).unwrap();
    }
    for sec in sections {
        file.write_all(bytes_of(&sec)).unwrap();
    }
    file.write_all(data).unwrap();
}
