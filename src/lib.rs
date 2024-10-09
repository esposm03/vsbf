use core::{fmt, str};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sym {
    pub name: u32, // offset into strtab
    pub size: u16,
    pub section: u16,
    pub value: u64,
}
impl Sym {
    pub fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, name) = number::le_u32(i)?;
        let (i, size) = number::le_u16(i)?;
        let (i, section) = number::le_u16(i)?;
        let (i, value) = number::le_u64(i)?;

        let ret = Self {
            name,
            size,
            section,
            value,
        };

        Ok((i, ret))
    }

    pub fn write(&self, w: &mut dyn io::Write) -> io::Result<()> {
        w.write_all(&self.name.to_le_bytes())?;
        w.write_all(&self.size.to_le_bytes())?;
        w.write_all(&self.section.to_le_bytes())?;
        w.write_all(&self.value.to_le_bytes())?;
        Ok(())
    }

    pub fn print(obj: &Vsbf, syms: &[Self]) {
        let mut name_len = 4;
        for sym in syms {
            name_len = obj.string_at(sym.name).len().max(name_len);
        }

        println!(
            "{:<name_len$} {:6} {:8} {}",
            "Name",
            "Size",
            "Value",
            "Section",
            name_len = name_len,
        );
        for sym in syms {
            println!(
                "{:<name_len$} 0x{:4x} 0x{:08x} {}",
                obj.string_at(sym.name),
                sym.size,
                sym.value,
                sym.section,
                name_len = name_len,
            );
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rel {
    pub typ: u16,
    pub addend: i16,
    pub needed: u32, // offset into strtab
    pub offset: u64,
}
impl Rel {
    pub fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, typ) = number::le_u16(i)?;
        let (i, addend) = number::le_i16(i)?;
        let (i, needed) = number::le_u32(i)?;
        let (i, offset) = number::le_u64(i)?;

        let ret = Self {
            typ,
            addend,
            needed,
            offset,
        };

        Ok((i, ret))
    }

    pub fn write(&self, w: &mut dyn io::Write) -> io::Result<()> {
        w.write_all(&self.typ.to_le_bytes())?;
        w.write_all(&self.addend.to_le_bytes())?;
        w.write_all(&self.needed.to_le_bytes())?;
        w.write_all(&self.offset.to_le_bytes())?;
        Ok(())
    }

    pub fn print(obj: &Vsbf, rels: &[Self]) {
        let mut name_len = 4;
        for rel in rels {
            name_len = 1 + obj.string_at(rel.needed).len().max(name_len);
        }

        println!(
            "{:<name_len$} {:6} {:8} {}",
            "Name",
            "Type",
            "Offset",
            "Addend",
            name_len = name_len,
        );
        for rel in rels {
            println!(
                "{:<name_len$} 0x{:4x} 0x{:08x} {}",
                obj.string_at(rel.needed),
                rel.typ,
                rel.offset,
                rel.addend,
                name_len = name_len,
            );
        }
    }
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
    pub typ: SectionType,
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
        let typ = SectionType::try_from(typ).unwrap();

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
                "{:4} {:4} {:08x} {:08x} {:08x}",
                hd.typ, hd.flags, hd.offset, hd.file_size, hd.memory,
            );
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SectionType {
    Text = 0,
    Data = 1,
    Rodata = 2,
}
impl TryFrom<u8> for SectionType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use SectionType::*;
        match value {
            0 => Ok(Text),
            1 => Ok(Data),
            2 => Ok(Rodata),
            _ => Err(value),
        }
    }
}
impl fmt::Display for SectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(match self {
            SectionType::Text => "text",
            SectionType::Data => "data",
            SectionType::Rodata => "rodata",
        })
    }
}
impl SectionType {
    pub fn to_le_bytes(&self) -> [u8; 1] {
        return [*self as u8];
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Vsbf {
    arch: u16,
    os: u16,
    segments: Vec<SegmentHeader>,
    sections: Vec<SectionHeader>,
    strtab: Vec<u8>,
    syms: Vec<Sym>,
    rels: Vec<Rel>,
    data: Vec<u8>,
}
impl Vsbf {
    pub fn empty() -> Self {
        Self {
            arch: 0,
            os: 0,
            segments: vec![],
            sections: vec![],
            strtab: vec![],
            syms: vec![],
            rels: vec![],
            data: vec![],
        }
    }

    pub fn parse(i: &[u8]) -> IResult<&[u8], Self> {
        let (i, header) = FileHeader::parse(i)?;

        let n_syms = header.num_symbols as usize;
        let n_segs = header.num_segments as usize;
        let n_rels = header.num_relocs as usize;
        let n_sects = header.num_sections as usize;

        let (i, segments) = multi::count(SegmentHeader::parse, n_segs)(i)?;
        let (i, sections) = multi::count(SectionHeader::parse, n_sects)(i)?;
        let (i, strtab) = bytes::take(header.strtab_size)(i)?;
        let (i, syms) = multi::count(Sym::parse, n_syms)(i)?;
        let (i, rels) = multi::count(Rel::parse, n_rels)(i)?;

        let file = Vsbf {
            arch: header.arch,
            os: header.os,
            segments,
            sections,
            strtab: strtab.to_vec(),
            syms,
            rels,
            data: i.to_vec(),
        };

        Ok((i, file))
    }

    pub fn write(&self, w: &mut dyn io::Write) -> io::Result<()> {
        let header = FileHeader {
            arch: self.arch,
            os: self.os,
            num_segments: self.segments.len() as _,
            num_sections: self.sections.len() as _,
            strtab_size: self.strtab.len() as _,
            num_symbols: self.syms.len() as _,
            num_relocs: self.rels.len() as _,
            next_header: 0,
        };

        header.write(w)?;
        for seg in &self.segments {
            seg.write(w)?;
        }
        for sec in &self.sections {
            sec.write(w)?;
        }
        w.write_all(&self.strtab)?;
        for sym in &self.syms {
            sym.write(w)?;
        }
        for rel in &self.rels {
            rel.write(w)?;
        }
        w.write_all(&self.data)?;

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
        self.segments.push(seg);
    }

    pub fn sections(&self) -> Vec<SectionHeader> {
        return self.sections.clone();
    }

    pub fn segments(&self) -> Vec<SegmentHeader> {
        self.segments.clone()
    }

    // === STRINGS ===

    pub fn push_string(&mut self, data: &str) {
        assert!(data.is_ascii());
        assert!(data.len() <= u16::MAX as usize);

        let len = data.len() as u16;

        self.strtab.extend_from_slice(&len.to_le_bytes());
        self.strtab.extend_from_slice(data.as_bytes());
    }

    pub fn strings(&self) -> StrTabIter {
        StrTabIter(self, 0)
    }

    pub fn string_at(&self, i: u32) -> &str {
        let i = i as usize;
        let len = u16::from_le_bytes(self.strtab[i..i + 2].try_into().unwrap()) as usize;
        assert!(i + 2 + len <= self.strtab.len());

        str::from_utf8(&self.strtab[i + 2..i + len + 2]).unwrap()
    }

    // === SYMBOLS ===

    pub fn push_sym(&mut self, sym: Sym) {
        self.syms.push(sym);

        // TODO: adjust all offsets
    }

    pub fn syms(&self) -> &[Sym] {
        &self.syms
    }

    pub fn syms_mut(&mut self) -> &mut [Sym] {
        &mut self.syms
    }

    // === RELOCATIONS ===

    pub fn rels(&self) -> &[Rel] {
        &self.rels
    }

    pub fn rels_mut(&mut self) -> &mut [Rel] {
        &mut self.rels
    }
}

pub struct StrTabIter<'a>(&'a Vsbf, usize);
impl<'a> Iterator for StrTabIter<'a> {
    type Item = (u32, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        let i = self.1;

        if i >= self.0.strtab.len() {
            return None;
        }

        let len = u16::from_le_bytes(self.0.strtab[i..i + 2].try_into().unwrap()) as usize;
        assert!(i + 2 + len <= self.0.strtab.len());

        self.1 += 2 + len;
        Some((
            i as u32,
            str::from_utf8(&self.0.strtab[i + 2..i + 2 + len]).unwrap(),
        ))
    }
}

#[test]
#[cfg(test)]
fn test_strtab() {
    let mut vsbf = Vsbf::empty();
    vsbf.push_string("hello");
    vsbf.push_string("");
    vsbf.push_string("hi");

    let mut iter = vsbf.strings();

    assert_eq!(iter.next().unwrap(), (0, "hello"));
    assert_eq!(iter.next().unwrap(), (7, ""));
    assert_eq!(iter.next().unwrap(), (9, "hi"));
    assert!(iter.next().is_none());

    assert_eq!(vsbf.string_at(0), "hello");
    assert_eq!(vsbf.string_at(7), "");
    assert_eq!(vsbf.string_at(9), "hi");
}

#[test]
#[cfg(test)]
#[should_panic]
fn test_strtab_malformed() {
    let vsbf = Vsbf {
        arch: 0,
        os: 0,
        segments: vec![],
        sections: vec![],
        strtab: vec![0x05, 0x00],
        data: vec![],
        rels: vec![],
        syms: vec![],
    };
    vsbf.strings().next();
}

#[test]
#[cfg(test)]
fn test_parse_write() {
    use std::io::Cursor;

    // Empty file
    let mut buf = Cursor::new(vec![]);
    let vsbf = Vsbf::empty();
    vsbf.write(&mut buf).unwrap();
    assert_eq!(Vsbf::parse(&buf.into_inner()).unwrap().1, vsbf);

    // File with a string and a symbol
    let mut buf = Cursor::new(vec![]);
    let mut vsbf = Vsbf::empty();
    vsbf.push_string("hello");
    vsbf.push_sym(Sym {
        name: 0,
        size: 10,
        section: 0,
        value: 20,
    });
    vsbf.write(&mut buf).unwrap();
    assert_eq!(Vsbf::parse(&buf.into_inner()).unwrap().1, vsbf);
}
