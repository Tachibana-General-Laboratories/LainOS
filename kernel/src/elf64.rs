//! ELF definitions common to all 64-bit architectures.

pub type Addr = u64;
pub type Half = u16;
pub type Off = u64;
pub type Sword = i32;
pub type Sxword = i64;
pub type Word = u32;
pub type Lword = u64;
pub type Xword = u64;

// Types of dynamic symbol hash table bucket and chain elements.
//
// This is inconsistent among 64 bit architectures, so a machine dependent
// pub is required.

pub type Hashelt = Word;

// Non-standard class-dependent datatype used for abstraction.
pub type Size = Xword;
pub type Ssize = Sxword;

/// ELF header.
pub struct Ehdr {
    /// File identification.
    pub e_ident: [u8, EI_NIDENT],
    /// File type.
    pub e_type: Half,
    /// Machine architecture.
    pub e_machine: Half,
    /// ELF format version.
    pub e_version: Word,
    /// Entry point.
    pub e_entry: Addr,
    /// Program header file offset.
    pub e_phoff: Off,
    /// Section header file offset.
    pub e_shoff: Off,
    /// Architecture-specific flags.
    pub e_flags: Word,
    /// Size of ELF header in bytes.
    pub e_ehsize: Half,
    /// Size of program header entry.
    pub e_phentsize: Half,
    /// Number of program header entries.
    pub e_phnum: Half,
    /// Size of section header entry.
    pub e_shentsize: Half,
    /// Number of section header entries.
    pub e_shnum: Half,
    /// Section name strings section.
    pub e_shstrndx: Half,
}

/// Shared object information, found in SHT_MIPS_LIBLIST.
pub struct Lib {
    /// The name of a shared object.
    pub l_name: Word,
    /// 64-bit timestamp.
    pub l_time_stamp: Word,
    /// Checksum of visible symbols, sizes.
    pub l_checksum: Word,
    /// Interface version string index.
    pub l_version: Word,
    /// Flags (LL_*).
    pub l_flags: Word,
}

/// Section header.
pub struct Shdr {
    /// Section name (index into the section header string table).
    pub sh_name: Word,
    /// Section type.
    pub sh_type: Word,
    /// Section flags.
    pub sh_flags: Xword,
    /// Address in memory image.
    pub sh_addr: Addr,
    /// Offset in file.
    pub sh_offset: Off,
    /// Size in bytes.
    pub sh_size: Xword,
    /// Index of a related section.
    pub sh_link: Word,
    /// Depends on section type.
    pub sh_info: Word,
    /// Alignment in bytes.
    pub sh_addralign: Xword,
    /// Size of each entry in section.
    pub sh_entsize: Xword,
}

/// Program header.
pub struct Phdr {
    /// Entry type.
    pub p_type: Word,
    /// Access permission flags.
    pub p_flags: Word,
    /// File offset of contents.
    pub p_offset: Off,
    /// Virtual address in memory image.
    pub p_vaddr: Addr,
    /// Physical address (not used).
    pub p_paddr: Addr,
    /// Size of contents in file.
    pub p_filesz: Xword,
    /// Size of contents in memory.
    pub p_memsz: Xword,
    /// Alignment in memory and file.
    pub p_align: Xword,
}

/// Dynamic structure.  The ".dynamic" section contains an array of them.
pub struct Dyn {
    /// Entry type.
    d_tag: Sxword,
    union {
        /// Integer value.
        d_val: Xword,
        /// Address value.
        d_ptr: Addr,
    } d_un,
}

// Relocation entries.

/// Relocations that don't need an addend field.
pub struct Rel {
    /// Location to be relocated.
    pub r_offset: Addr,
    /// Relocation type and symbol index.
    pub r_info: Xword,
}

/// Relocations that need an addend field.
pub struct Rela {
    /// Location to be relocated.
    pub r_offset: Addr,
    /// Relocation type and symbol index.
    pub r_info: Xword,
    /// Addend.
    pub r_addend: Sxword,
}

// Macros for accessing the fields of r_info.
#define	ELF64_R_SYM(info)	((info) >> 32)
#define	ELF64_R_TYPE(info)	((info) & 0xffffffffL)

// Macro for constructing r_info from field values.
#define	ELF64_R_INFO(sym, type)	(((sym) << 32) + ((type) & 0xffffffffL))

#define	ELF64_R_TYPE_DATA(info)	(((Xword)(info)<<32)>>40)
#define	ELF64_R_TYPE_ID(info)	(((Xword)(info)<<56)>>56)
#define	ELF64_R_TYPE_INFO(data, type)	\
                (((Xword)(data)<<8)+(Xword)(type))

/// Note entry header
pub type Elf_Note = Nhdr;

/// Move entry
pub struct Move {
    /// symbol value
    pub m_value: Lword,
    /// size + index
    pub m_info: Xword,
    /// symbol offset
    pub m_poffset: Xword,
    /// repeat count
    pub m_repeat: Half,
    /// stride info
    pub m_stride: Half,
}

#define ELF64_M_SYM(info)	((info)>>8)
#define ELF64_M_SIZE(info)	((unsigned char)(info))
#define ELF64_M_INFO(sym, size)	(((sym)<<8)+(unsigned char)(size))

/// Hardware/Software capabilities entry
pub struct Cap {
    /// how to interpret value
    c_tag: Xword,
    union {
        c_val: Xword,
        c_ptr: Addr,
    } c_un,
}

/// Symbol table entries.
pub struct Sym {
    /// String table index of name.
    pub st_name: Word,
    /// Type and binding information.
    pub st_info: u8,
    /// Reserved (not used).
    pub st_other: u8,
    /// Section index of symbol.
    pub st_shndx: Half,
    /// Symbol value.
    pub st_value: Addr,
    /// Size of associated object.
    pub st_size: Xword,
}

// Macros for accessing the fields of st_info.
#define	ELF64_ST_BIND(info)		((info) >> 4)
#define	ELF64_ST_TYPE(info)		((info) & 0xf)

// Macro for constructing st_info from field values.
#define	ELF64_ST_INFO(bind, type)	(((bind) << 4) + ((type) & 0xf))

// Macro for accessing the fields of st_other.
#define	ELF64_ST_VISIBILITY(oth)	((oth) & 0x3)

/// Structures used by Sun & GNU-style symbol versioning.
pub struct Verdef {
    pub vd_version: Half,
    pub vd_flags: Half,
    pub vd_ndx: Half,
    pub vd_cnt: Half,
    pub vd_hash: Word,
    pub vd_aux: Word,
    pub vd_next: Word,
}

pub struct Verdaux {
    pub vda_name: Word,
    pub vda_next: Word,
}

pub struct Verneed {
    pub vn_version: Half,
    pub vn_cnt: Half,
    pub vn_file: Word,
    pub vn_aux: Word,
    pub vn_next: Word,
}

pub struct Vernaux {
    pub vna_hash: Word,
    pub vna_flags: Half,
    pub vna_other: Half,
    pub vna_name: Word,
    pub vna_next: Word,
}

pub type Versym = Half;

pub struct Syminfo {
    /// direct bindings - symbol bound to
    pub si_boundto: Half,
    /// per symbol flags
    pub si_flags: Half,
}

pub struct Chdr {
    pub ch_type: Word,
    pub ch_reserved: Word,
    pub ch_size: Xword,
    pub ch_addralign: Xword,
}
