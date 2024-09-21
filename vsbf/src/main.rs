use std::env::args;

use capstone::{
    arch::{ArchDetail, BuildsCapstone, BuildsCapstoneSyntax},
    Capstone, InsnDetail, InsnGroupId, RegId,
};
use vsbf::{parse_file, print_header, print_section_headers, print_segment_headers};

fn main() {
    let buf = std::fs::read(args().nth(1).unwrap()).unwrap();
    let (i, file) = parse_file(&buf).unwrap();

    println!("Remaining data: {:?}", i);

    print_header(&file.header);
    println!();
    print_segment_headers(&file.segments);
    println!();
    print_section_headers(&file.sections);

    let cs = Capstone::new()
        .x86()
        .mode(capstone::arch::x86::ArchMode::Mode64)
        .syntax(capstone::arch::x86::ArchSyntax::Intel)
        .detail(true)
        .build()
        .unwrap();

    let start = file.sections[0].offset as usize;
    let end = start + file.sections[0].file_size as usize;

    println!("{:x?}", &buf[start..end]);

    let insns = cs
        .disasm_all(&buf[start..end], 0x1000)
        .expect("Failed to disassemble");
    println!("Found {} instructions", insns.len());
    for i in insns.as_ref() {
        println!();
        println!("{}", i);

        let detail: InsnDetail = cs.insn_detail(&i).expect("Failed to get insn detail");
        let arch_detail: ArchDetail = detail.arch_detail();
        let ops = arch_detail.operands();

        let output: &[(&str, String)] = &[
            ("insn id:", format!("{:?}", i.id().0)),
            ("bytes:", format!("{:?}", i.bytes())),
            ("read regs:", reg_names(&cs, detail.regs_read())),
            ("write regs:", reg_names(&cs, detail.regs_write())),
            ("insn groups:", group_names(&cs, detail.groups())),
        ];

        for &(ref name, ref message) in output.iter() {
            println!("{:4}{:12} {}", "", name, message);
        }

        println!("{:4}operands: {}", "", ops.len());
        for op in ops {
            println!("{:8}{:?}", "", op);
        }
    }
}

/// Print register names
fn reg_names(cs: &Capstone, regs: &[RegId]) -> String {
    let names: Vec<String> = regs.iter().map(|&x| cs.reg_name(x).unwrap()).collect();
    names.join(", ")
}

/// Print instruction group names
fn group_names(cs: &Capstone, regs: &[InsnGroupId]) -> String {
    let names: Vec<String> = regs.iter().map(|&x| cs.group_name(x).unwrap()).collect();
    names.join(", ")
}
