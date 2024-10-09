use std::{env::args, ffi::OsStr, os::unix::ffi::OsStrExt, u64};

use capstone::{
    arch::{
        x86::{ArchMode, ArchSyntax},
        BuildsCapstone, BuildsCapstoneSyntax,
    },
    Capstone,
};
use unicorn_engine::{
    Arch,
    InsnSysX86::SYSCALL,
    Mode, Permission,
    RegisterX86::{RAX, RDI, RDX, RSI},
    Unicorn, SECOND_SCALE,
};
use vsbf::Vsbf;

fn main() {
    let buf = std::fs::read(args().nth(1).unwrap()).unwrap();
    let (data, file) = Vsbf::parse(&buf).unwrap();

    let cs = Capstone::new()
        .x86()
        .mode(ArchMode::Mode64)
        .syntax(ArchSyntax::Intel)
        .detail(true)
        .build()
        .expect("Failed to initialize Capstone");
    let mut emu = Unicorn::new(Arch::X86, Mode::MODE_64).expect("Failed to init Unicorn");

    for segment in file.segments() {
        emu.mem_map(
            segment.mem + 0x1000,
            (segment.mem_size as usize & 0x1000) + 0x1000,
            Permission::from_bits(segment.flags.bits() as _).unwrap(),
        )
        .unwrap();

        let start = segment.file as usize;
        let end = start + segment.file_size as usize;
        emu.mem_write(segment.mem + 0x1000, &data[start..end])
            .unwrap();
    }

    // Disassemble every instruction
    let code = emu.mem_read_as_vec(0x1000, 0x1000).unwrap();
    emu.add_code_hook(0, u64::MAX, move |_, addr, _| {
        let disasm = cs
            .disasm_count(&code[addr as usize - 0x1000..], addr, 1)
            .unwrap();

        print!("{}", disasm);
    })
    .unwrap();

    emu.add_insn_sys_hook(SYSCALL, 0x1000, u64::MAX, syscall)
        .unwrap();

    emu.emu_start(
        0x1000,
        file.segments()[0].file_size as u64 + 0x1000,
        10 * SECOND_SCALE,
        1000,
    )
    .unwrap();
}

fn syscall(emu: &mut Unicorn<'_, ()>) {
    let rax = emu.reg_read(RAX).unwrap();
    let rdi = emu.reg_read(RDI).unwrap();
    let rsi = emu.reg_read(RSI).unwrap();
    let rdx = emu.reg_read(RDX).unwrap();

    match rax {
        1 => {
            let buf = emu.mem_read_as_vec(rsi, rdx as _).unwrap();
            let buf = OsStr::from_bytes(&buf);
            println!("SYSCALL: write({rdi}, {buf:?}, {rdx})")
        }
        0x3c => {
            println!("SYSCALL: exit({rdi})");
            emu.emu_stop().unwrap();
        }
        _ => {
            println!("SYSCALL {rax:x} {rdi:x} {rsi:x} {rdx:x}");
            todo!()
        }
    }
}
