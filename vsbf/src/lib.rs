use std::fmt::Display;

use bitflags::bitflags;
use bytemuck::{Pod, Zeroable};
use nom::{bytes::complete as bytes, multi, number::complete as number, IResult};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Zeroable, Pod)]
pub struct Header {
    pub arch: u16,
    pub os: u16,
    pub num_segments: u16,
    pub num_sections: u16,
    pub next_header: u64,
}

bitflags! {
    /// The access restrictions the given segment will have when loaded in memory
    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Zeroable, Pod)]
    pub struct PermissionFlags: u8 {
        /// The segment is readable
        const R = 0b1;
        /// The segment is writable
        const W = 0b10;
        /// The segment is executable
        const X = 0b100;
    }
}
impl Display for PermissionFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(&format!(
            "{}{}{}",
            if self.contains(Self::R) { "R" } else { " " },
            if self.contains(Self::W) { "W" } else { " " },
            if self.contains(Self::X) { "X" } else { " " }
        ))
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Zeroable, Pod)]
pub struct SegmentHeader {
    pub typ: u8,
    pub flags: PermissionFlags,
    pub align: u16,
    pub file: u32,
    pub mem: u64,
    pub file_size: u32,
    pub mem_size: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Zeroable, Pod)]
pub struct SectionHeader {
    pub typ: u8,
    pub flags: PermissionFlags,
    pub file_size: u16,
    pub offset: u32,
    pub memory: u64,
}

pub struct Vsbf<'a> {
    pub header: Header,
    pub sections: Vec<SectionHeader>,
    pub segments: Vec<SegmentHeader>,
    pub data: &'a [u8],
}

pub fn parse_header(i: &[u8]) -> IResult<&[u8], Header> {
    let (i, _) = bytes::tag("VSBF")(i)?;
    let (i, arch) = number::le_u16(i)?;
    let (i, os) = number::le_u16(i)?;
    let (i, num_segments) = number::le_u16(i)?;
    let (i, num_sections) = number::le_u16(i)?;
    let (i, next_header) = number::le_u64(i)?;

    let ret = Header {
        arch,
        os,
        num_segments,
        num_sections,
        next_header,
    };

    Ok((i, ret))
}

pub fn print_header(hd: &Header) {
    println!("Architecture: {}", hd.arch);
    println!("OS/ABI: {}", hd.os);
    println!("Number of segment headers: {}", hd.num_segments);
    println!("Number of section headers: {}", hd.num_sections);
    println!("Next file header: {}", hd.next_header);
}

pub fn parse_segment_header(i: &[u8]) -> IResult<&[u8], SegmentHeader> {
    let (i, typ) = number::le_u8(i)?;
    let (i, flags) = number::le_u8(i)?;
    let (i, align) = number::le_u16(i)?;
    let (i, file) = number::le_u32(i)?;
    let (i, mem) = number::le_u64(i)?;
    let (i, file_size) = number::le_u32(i)?;
    let (i, mem_size) = number::le_u32(i)?;

    let flags = PermissionFlags::from_bits_truncate(flags);

    let ret = SegmentHeader {
        typ,
        flags,
        align,
        file,
        mem,
        file_size,
        mem_size,
    };

    Ok((i, ret))
}

pub fn print_segment_headers(hd: &[SegmentHeader]) {
    println!(
        "{:<typ_len$} {:4} {:10} {:10} {:10} {:10} Align",
        "Type",
        "Flag",
        "File",
        "FileSize",
        "Mem",
        "MemSize",
        typ_len = 4,
    );
    for hd in hd {
        println!(
            "{:<typ_len$} {:4} 0x{:08x} 0x{:08x} 0x{:08x} 0x{:08x} 0x{:x}",
            "LOAD",
            hd.flags,
            hd.file,
            hd.file_size,
            hd.mem,
            hd.mem_size,
            hd.align,
            typ_len = 4,
        );
    }
}

pub fn parse_section_header(i: &[u8]) -> IResult<&[u8], SectionHeader> {
    let (i, typ) = number::le_u8(i)?;
    let (i, flags) = number::le_u8(i)?;
    let (i, file_size) = number::le_u16(i)?;
    let (i, offset) = number::le_u32(i)?;
    let (i, memory) = number::le_u64(i)?;

    let flags = PermissionFlags::from_bits_truncate(flags);

    let ret = SectionHeader {
        typ,
        flags,
        offset,
        file_size,
        memory,
    };

    Ok((i, ret))
}

pub fn print_section_headers(hd: &[SectionHeader]) {
    println!(
        "{:4} {:4} {:8} {:8} {}",
        "Type", "Flag", "Offset", "Size", "Address"
    );

    for hd in hd {
        println!(
            "{:4x} {:4} {:08x} {:08x} {:08x}",
            hd.typ, hd.flags, hd.offset, hd.file_size, hd.memory,
        );
    }
}

pub fn parse_file<'a>(i: &'a [u8]) -> IResult<&'a [u8], Vsbf<'a>> {
    let (i, header) = parse_header(i)?;
    let (i, segments) = multi::many_m_n(0, header.num_segments as usize, parse_segment_header)(i)?;
    let (i, sections) = multi::many_m_n(0, header.num_sections as usize, parse_section_header)(i)?;

    let file = Vsbf {
        header,
        sections,
        segments,
        data: i,
    };

    Ok((i, file))
}
