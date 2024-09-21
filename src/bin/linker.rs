use std::{env::args, fs, process::exit};

use multimap::MultiMap;
use vsbf::Vsbf;

fn main() {
    if args().len() < 2 {
        eprintln!("Usage: {} <filename> [filename...]", args().next().unwrap());
        exit(1);
    }

    let filenames: Vec<String> = args().skip(1).collect();
    let files: Vec<Vsbf> = filenames
        .iter()
        .map(|filename| Vsbf::parse(&fs::read(&filename).unwrap()).unwrap().1)
        .collect();

    // TODO: check that there are no relocations referring to a name without a symbol defined (aka, check for undefined symbols)
    // TODO: check that every relocation is in bounds of a single section

    merge_strtabs(&files);

    // let mut objects = Vec::with_capacity(args().len() - 1);
    // let sections = obj.sections();

    // let start = sections[0].offset as usize;
    // let end = start + sections[0].file_size as usize;
    // println!("{:x?}", &buf[start..end]);

    // obj.add_segment(SegmentHeader {
    //     typ: 0,
    //     flags: PermissionFlags::all(),
    //     align: 0x1000,
    //     file: start as u32,
    //     mem: 0,
    //     file_size: sections[0].file_size as _,
    //     mem_size: sections[0].file_size as _,
    // });

    // let mut file = File::create(filename.trim_end_matches(".o")).unwrap();
    // obj.write(&mut file).unwrap();
}

fn merge_strtabs(objs: &[Vsbf]) -> Vec<u8> {
    let mut duplicates = MultiMap::new();

    for (i, obj) in objs.iter().enumerate() {
        for (j, str) in obj.strings() {
            duplicates.insert(str, (i, j));
        }
    }

    let mut new_strtab = vec![];

    for (str, _entries) in duplicates {
        let _off = new_strtab.len();

        let len: u16 = str.len().try_into().unwrap();
        new_strtab.extend_from_slice(&len.to_le_bytes());
        new_strtab.extend_from_slice(str.as_bytes());

        // TODO: adjust symbols and relocations
    }

    new_strtab
}
