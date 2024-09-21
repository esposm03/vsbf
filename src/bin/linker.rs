use std::{env::args, fs, process::exit};

use vsbf::Vsbf;

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

    // TODO: check that there are no relocations referring to a name without a symbol defined (aka, check for undefined symbols)
    // TODO: check that every relocation is in bounds of a single section

    merge_strtabs(&mut files);

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

    for (off, i, j) in dupes {
        for sym in objs[i].syms_mut() {
            if sym.name == j {
                sym.name = off;
            }
        }
    }

    // TODO: adjust relocations

    new_strtab
}

#[cfg(test)]
mod tests {
    use vsbf::Sym;

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
}
