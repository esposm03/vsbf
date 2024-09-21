use std::{fmt::Display, io};

use bitflags::bitflags;
use nom::{bytes::complete as bytes, multi, number::complete as number, IResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FileHeader {
    pub arch: u16,
    pub os: u16,
    pub num_segments: u16,
    pub num_sections: u16,
    pub strtab_size: u32,
    pub num_symbols: u32,
    pub num_relocs: u32,
    pub next_header: u64,
}
impl FileHeader {
    pub fn parse(i: &[u8]) -> IResult<&[u8], FileHeader> {
        let (i, _) = bytes::tag("VSBF")(i)?;
        let (i, arch) = number::le_u16(i)?;
        let (i, os) = number::le_u16(i)?;
        let (i, num_segments) = number::le_u16(i)?;
        let (i, num_sections) = number::le_u16(i)?;
        let (i, strtab_size) = number::le_u32(i)?;
        let (i, num_symbols) = number::le_u32(i)?;
        let (i, num_relocs) = number::le_u32(i)?;
        let (i, next_header) = number::le_u64(i)?;

        let ret = FileHeader {
            arch,
            os,
            num_segments,
            num_sections,
            next_header,
            strtab_size,
            num_symbols,
            num_relocs,
        };

        Ok((i, ret))
    }

    pub fn write(&self, w: &mut dyn io::Write) -> io::Result<()> {
        w.write_all(b"VSBF")?;
        w.write_all(&self.arch.to_le_bytes())?;
        w.write_all(&self.os.to_le_bytes())?;
        w.write_all(&self.num_segments.to_le_bytes())?;
        w.write_all(&self.num_sections.to_le_bytes())?;
        w.write_all(&self.strtab_size.to_le_bytes())?;
        w.write_all(&self.num_symbols.to_le_bytes())?;
        w.write_all(&self.num_relocs.to_le_bytes())?;
        w.write_all(&self.next_header.to_le_bytes())?;
        Ok(())
    }

    pub fn print(&self) {
        println!("Architecture: {}", self.arch);
        println!("OS/ABI: {}", self.os);
        println!("Number of segment headers: {}", self.num_segments);
        println!("Number of section headers: {}", self.num_sections);
        println!("Next file header: {}", self.next_header);
    }
}

pub struct Sym {
    pub name: u32, // offset into strtab
    pub size: u16,
    pub section: u16,
    pub value: u64,
}

pub struct Rel {
    pub typ: u8,
    pub needed: u32, // offset into strtab
    pub address: u64,
}

bitflags! {
    /// The access restrictions the given segment will have when loaded in memory
    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

pub const SEGMENT_HDR_SIZE: u32 = 24;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SegmentHeader {
    pub typ: u8,
    pub flags: PermissionFlags,
    pub align: u16,
    pub file: u32,
    pub mem: u64,
    pub file_size: u32,
    pub mem_size: u32,
}
impl SegmentHeader {
    pub fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, typ) = number::le_u8(i)?;
        let (i, flags) = number::le_u8(i)?;
        let (i, align) = number::le_u16(i)?;
        let (i, file) = number::le_u32(i)?;
        let (i, mem) = number::le_u64(i)?;
        let (i, file_size) = number::le_u32(i)?;
        let (i, mem_size) = number::le_u32(i)?;

        let flags = PermissionFlags::from_bits_truncate(flags);

        let ret = Self {
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

    pub fn write(&self, w: &mut dyn io::Write) -> io::Result<()> {
        w.write_all(&self.typ.to_le_bytes())?;
        w.write_all(&self.flags.bits().to_le_bytes())?;
        w.write_all(&self.align.to_le_bytes())?;
        w.write_all(&self.file.to_le_bytes())?;
        w.write_all(&self.mem.to_le_bytes())?;
        w.write_all(&self.file_size.to_le_bytes())?;
        w.write_all(&self.mem_size.to_le_bytes())?;
        Ok(())
    }

    pub fn print(hd: &[Self]) {
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
}

pub const SECTION_HDR_SIZE: u32 = 16;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SectionHeader {
    pub typ: u8,
    pub flags: PermissionFlags,
    pub file_size: u16,
    pub offset: u32,
    pub memory: u64,
}
impl SectionHeader {
    pub fn parse(i: &[u8]) -> IResult<&[u8], SectionHeader> {
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

    pub fn write(&self, w: &mut dyn io::Write) -> io::Result<()> {
        w.write_all(&self.typ.to_le_bytes())?;
        w.write_all(&self.flags.bits().to_le_bytes())?;
        w.write_all(&self.file_size.to_le_bytes())?;
        w.write_all(&self.offset.to_le_bytes())?;
        w.write_all(&self.memory.to_le_bytes())?;
        Ok(())
    }

    pub fn print(hd: &[SectionHeader]) {
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
}

pub struct Vsbf<'a> {
    arch: u16,
    os: u16,
    segments: Vec<SegmentHeader>,
    sections: Vec<SectionHeader>,
    data: &'a [u8],
}
impl<'a> Vsbf<'a> {
    pub fn parse(i: &'a [u8]) -> IResult<&'a [u8], Self> {
        let (i, header) = FileHeader::parse(i)?;
        let (i, segments) =
            multi::many_m_n(0, header.num_segments as usize, SegmentHeader::parse)(i)?;
        let (i, sections) =
            multi::many_m_n(0, header.num_sections as usize, SectionHeader::parse)(i)?;

        let file = Vsbf {
            os: header.os,
            arch: header.arch,
            sections,
            segments,
            data: i,
        };

        Ok((i, file))
    }

    pub fn write(&self, w: &mut dyn io::Write) -> io::Result<()> {
        let header = FileHeader {
            arch: self.arch,
            os: self.os,
            num_segments: self.segments.len() as _,
            num_sections: self.sections.len() as _,
            strtab_size: 0,
            num_symbols: 0,
            num_relocs: 0,
            next_header: 0,
        };

        header.write(w)?;
        for seg in &self.segments {
            seg.write(w)?;
        }
        for sec in &self.sections {
            sec.write(w)?;
        }
        // write other things
        w.write_all(self.data)?;

        Ok(())
    }

    pub fn add_segment(&mut self, mut seg: SegmentHeader) {
        seg.file += SEGMENT_HDR_SIZE;
        for segm in &mut self.segments {
            segm.file += SEGMENT_HDR_SIZE;
        }
        for sect in &mut self.sections {
            sect.offset += SEGMENT_HDR_SIZE;
        }
    }

    pub fn sections(&self) -> Vec<SectionHeader> {
        return self.sections.clone();
    }
}
