use std::{
    collections::HashMap,
    env::args,
    fs::{self, File},
    process::exit,
};

use vsbf::{PermissionFlags, SectionHeader, SegmentHeader, Vsbf};

fn main() {
    if args().len() < 2 {
        eprintln!("Usage: {} <filename> [filename...]", args().next().unwrap());
        exit(1);
    }

    let filenames: Vec<String> = args().skip(1).collect();
    let mut files: Vec<Vsbf> = filenames
        .iter()
        .map(|filename| Vsbf::parse(&fs::read(&filename).unwrap()).unwrap().1)
        .collect();

    let sections: Vec<_> = files.iter().flat_map(|f| f.sections()).collect();
    println!("Sections: {sections:?}");
    let start = 0;

    // TODO: check that there are no relocations referring to a name without a symbol defined (aka, check for undefined symbols)
    // TODO: check that every relocation is in bounds of a single section
    let mut output = Vsbf::empty();

    output.set_strtab(merge_strtabs(&mut files));
    merge_sections(&mut output, &mut files);

    output.add_segment(SegmentHeader {
        typ: 0,
        flags: PermissionFlags::all(),
        align: 0x1000,
        file: start as u32,
        mem: 0,
        file_size: sections[0].file_size as _,
        mem_size: sections[0].file_size as _,
    });

    let mut outfile = File::create("test").unwrap();
    output.write(&mut outfile).unwrap();
}

fn merge_strtabs(objs: &mut [Vsbf]) -> Vec<u8> {
    let mut new_strtab = vec![];
    let mut dupes = vec![];

    for (i, obj) in objs.iter().enumerate() {
        for (j, str) in obj.strings() {
            let off = new_strtab.len() as u32;

            let len: u16 = str.len().try_into().unwrap();
            new_strtab.extend_from_slice(&len.to_le_bytes());
            new_strtab.extend_from_slice(str.as_bytes());

            dupes.push((off, i, j));
        }
    }

    // Adjust symbols
    for &(off, i, j) in &dupes {
        for sym in objs[i].syms_mut() {
            if sym.name == j {
                sym.name = off;
            }
        }
    }

    // TODO: Adjust relocations

    new_strtab
}

fn merge_sections(out: &mut Vsbf, objs: &mut [Vsbf]) {
    let mut merged = HashMap::<_, (SectionHeader, Vec<u8>)>::new();

    for obj in objs {
        for sec in obj.sections() {
            let start = sec.offset as usize;
            let end = start + sec.file_size as usize;

            if let Some((x, data)) = merged.get_mut(&sec.typ) {
                x.file_size += sec.file_size;
                x.flags |= sec.flags;
                data.extend_from_slice(&obj.data()[start..end]);
            } else {
                merged.insert(sec.typ, (sec, obj.data()[start..end].to_vec()));
            }
        }
    }

    let mut merged: Vec<_> = merged.into_values().collect();
    merged.sort_unstable_by_key(|s| s.0.typ as u8);
    for (mut sec, data) in merged {
        sec.offset = out.data().len() as u32;
        out.push_section(sec);
        out.data_mut().extend_from_slice(&data);
    }
}

#[cfg(test)]
mod tests {
    use vsbf::{SectionType, Sym};

    use super::*;

    #[test]
    fn test_merge_strtab() {
        let mut v1 = Vsbf::empty();
        let mut v2 = Vsbf::empty();

        v1.push_string("hello");
        v1.push_sym(Sym {
            name: 0,
            size: 0,
            section: 0,
            value: 0,
        });

        v2.push_string("Hi");
        v2.push_sym(Sym {
            name: 0,
            size: 0,
            section: 0,
            value: 0,
        });

        let mut objs = [v1, v2];
        let new_strtab = merge_strtabs(&mut objs);
        let [v1, v2] = objs;

        assert_eq!(new_strtab, b"\x05\x00hello\x02\x00Hi");
        assert_eq!(v1.syms()[0].name, 0);
        assert_eq!(v2.syms()[0].name, 7);
    }

    #[test]
    fn test_merge_sections() {
        let mut v1 = Vsbf::empty();
        let mut v2 = Vsbf::empty();
        let mut out = Vsbf::empty();

        v1.push_section(SectionHeader {
            typ: SectionType::Text,
            flags: PermissionFlags::R | PermissionFlags::X,
            file_size: 10,
            offset: 0,
            memory: 0,
        });
        v1.data_mut().extend_from_slice(b"aaaaabbbbb");
        v1.push_section(SectionHeader {
            typ: SectionType::Rodata,
            flags: PermissionFlags::R,
            file_size: 10,
            offset: 10,
            memory: 0,
        });
        v1.data_mut().extend_from_slice(b"cccccddddd");
        v1.push_section(SectionHeader {
            typ: SectionType::Rodata,
            flags: PermissionFlags::R,
            file_size: 10,
            offset: 20,
            memory: 0,
        });
        v1.data_mut().extend_from_slice(b"eeeeefffff");

        v2.push_section(SectionHeader {
            typ: SectionType::Text,
            flags: PermissionFlags::R | PermissionFlags::X,
            file_size: 20,
            offset: 0,
            memory: 0,
        });
        v2.data_mut().extend_from_slice(b"ggggghhhhhiiiiijjjjj");
        v2.push_section(SectionHeader {
            typ: SectionType::Data,
            flags: PermissionFlags::R | PermissionFlags::W,
            file_size: 10,
            offset: 20,
            memory: 0,
        });
        v2.data_mut().extend_from_slice(b"kkkkklllll");

        let mut objs = [v1, v2];
        merge_sections(&mut out, &mut objs);

        assert!(out.sections()[0].is_text());
        assert!(out.sections()[0].is_rx());
        assert_eq!(out.sections()[0].offset, 0);
        assert_eq!(out.sections()[0].file_size, 30);
        assert_eq!(&out.data()[0..30], b"aaaaabbbbbggggghhhhhiiiiijjjjj");

        assert!(out.sections()[1].is_data());
        assert!(out.sections()[1].is_rw());
        assert_eq!(out.sections()[1].offset, 30);
        assert_eq!(out.sections()[1].file_size, 10);
        assert_eq!(&out.data()[30..40], b"kkkkklllll");

        println!("{:?}", out.sections()[2]);
        assert!(out.sections()[2].is_rodata());
        assert!(out.sections()[2].is_ronly());
        assert_eq!(out.sections()[2].offset, 40);
        assert_eq!(out.sections()[2].file_size, 20);
        assert_eq!(&out.data()[40..60], b"cccccdddddeeeeefffff");
    }
}
