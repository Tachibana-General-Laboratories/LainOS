#![allow(unused_imports)]
use core::slice::from_raw_parts;
use core::mem::size_of;
use alloc::Vec;
use alloc::string::ToString;
use vfat::traits::File;
use sys::io;
use sys::VecExt;

use vm::{self, VirtualAddr};

const SIZEOF_IDENT: usize = 16;

// Values for Proghdr type
pub const PROG_LOAD: u32 = 1;

bitflags! {
    pub struct ProgramFlags: u32 {
        const R = 1 << 2;
        const W = 1 << 1;
        const X = 1 << 0;
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct Header {
    pub e_ident: [u8; SIZEOF_IDENT],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

impl Header {
    pub const SIZEOF: usize = size_of::<Self>();

    pub fn from_bytes(bytes: &[u8; Self::SIZEOF]) -> &Self {
        unsafe { &*(bytes.as_ptr() as *const Self) }
    }

    pub fn check_magic(&self) -> bool {
        //unimplemented!("elf::Header::check_magic")
        true
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

impl ProgramHeader {
    pub const SIZEOF: usize = size_of::<Self>();

    pub fn from_bytes(bytes: &[u8; Self::SIZEOF]) -> &Self {
        unsafe { &*(bytes.as_ptr() as *const Self) }
    }

    /*
    pub fn from_bytes(bytes: &[u8]) -> &[Self] {
        let len = bytes.len() & Self::SIZEOF;
        let ptr = bytes.as_ptr() as *const Self;
        unsafe {
            from_raw_parts(ptr, len)
        }
    }
    */

    pub fn is_loadable(&self) -> bool {
        self.p_type == PROG_LOAD
    }

    pub fn flags(&self) -> ProgramFlags {
        ProgramFlags::from_bits_truncate(self.p_flags)
    }

    pub fn is_read(&self) -> bool {
        self.flags().contains(ProgramFlags::R)
    }
    pub fn is_write(&self) -> bool {
        self.flags().contains(ProgramFlags::W)
    }
    pub fn is_executable(&self) -> bool {
        self.flags().contains(ProgramFlags::X)
    }

    pub fn check_vaddr(&self, align: usize) -> bool {
        (self.p_vaddr % align as u64) != 0 &&
        self.p_vaddr + self.p_memsz < self.p_vaddr
    }

    /*
    pub fn area(&self) -> vm::Area {
        let start = self.p_vaddr;
        let end = self.p_vaddr + self.p_memsz;
        vm::Area {
            start: VirtualAddr::from(start as *mut u8),
            end: VirtualAddr::from(end as *mut u8),
            readable: self.is_read(),
            writable: self.is_write(),
            executable: self.is_executable(),
        }
    }
    */

    /*
    pub fn file_range(&self) -> (usize, usize) {
    }
    pub fn vm_range(&self) -> (usize, usize) {
    }
    */
}

/*

pub fn spawn<F: File>(file: &mut F, aux: &[u8]) -> io::Result<()> {
    //start addr: 0x80000

    let mut header = [0u8; Header::SIZEOF];
    file.read_exact(&mut header)?;
    let header = Header::from_bytes(&header);


    if !header.check_magic() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "spawn: bad magic"))
    }

    let _ = file.seek(io::SeekFrom::Start(header.e_phoff))?;
    let mut data: Vec<u8> = vec![0; ProgramHeader::SIZEOF * header.e_phnum as usize];
    file.read_exact(&mut data)?;

    let phdrs: Vec<ProgramHeader> = unsafe { data.cast() };

    let vm = vm::Memory::with_capacity(phdrs.len());

    for ph in phdrs {
        if !ph.is_loadable() {
            continue;
        }
        if ph.check_vaddr(PAGESZ) && ph.p_memsz < ph.p_filesz {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "spawn: fail mem attributes"))
        }

        /*
        vm.add(vm::Area {
            start: VirtualAddr::from(ph.p_vaddr as *mut u8),
            end: VirtualAddr::from((ph.p_vaddr + ph.p_memsz) as *mut u8),
            readable: ph.is_readable(),
            writable: ph.is_writable(),
            executable: ph.is_executable(),
        });
        */


        /*
        if((sz = allocuvm(pgdir, sz, ph.vaddr + ph.memsz)) == 0) goto bad;
        if(ph.vaddr % PGSIZE != 0) goto bad;
        if(loaduvm(pgdir, (char*)ph.vaddr, ip, ph.off, ph.filesz) < 0)
        */
    }

    Ok(())
}
*/
