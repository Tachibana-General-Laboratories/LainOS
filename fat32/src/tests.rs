use std::io::prelude::*;
use std::io::Cursor;
use std::path::Path;

use vfat::{Shared, VFat, BiosParameterBlock};
use mbr::{MasterBootRecord, CHS, PartitionEntry};
use traits::*;

macro check_size($T:ty, $size:expr) {
    assert_eq!(::std::mem::size_of::<$T>(), $size,
        "'{}' does not have the expected size of {}", stringify!($T), $size);
}

macro expect_variant($e:expr, $variant:pat $(if $($cond:tt)*)*) {
    match $e {
        $variant $(if $($cond)*)* => {  },
        o => panic!("expected '{}' but found '{:?}'", stringify!($variant), o)
    }
}

macro resource($name:expr) {{
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../files/resources/", $name);
    match ::std::fs::File::open(path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("\nfailed to find assignment 2 resource '{}': {}\n\
                       => perhaps you need to run 'make fetch'?", $name, e);
            panic!("missing resource");
        }
    }
}}

macro assert_hash_eq($name:expr, $actual:expr, $expected:expr) {
    let (actual, expected) = ($actual, $expected);
    let (actual, expected) = (actual.trim(), expected.trim());
    if actual != expected {
        eprintln!("\nFile system hash failed for {}!\n", $name);
        eprintln!("--------------- EXPECTED ---------------");
        eprintln!("{}", expected);
        eprintln!("---------------- ACTUAL ----------------");
        eprintln!("{}", actual);
        eprintln!("---------------- END ----------------");
        panic!("hash mismatch")
    }
}

macro hash_for($name:expr) {{
    let mut file = resource!(concat!("hashes/", $name));
    let mut string = String::new();
    file.read_to_string(&mut string).expect("read hash to string");
    string
}}

macro vfat_from_resource($name:expr) {
    VFat::from(resource!($name)).expect("failed to initialize VFAT from image")
}

#[test]
fn check_mbr_size() {
    check_size!(MasterBootRecord, 512);
    check_size!(PartitionEntry, 16);
    check_size!(CHS, 3);
}

#[test]
fn check_mbr_signature() {
    let mut data = [0u8; 512];
    let e = MasterBootRecord::from(Cursor::new(&mut data[..])).unwrap_err();
    expect_variant!(e, ::mbr::Error::BadSignature);

    data[510..].copy_from_slice(&[0x55, 0xAA]);
    MasterBootRecord::from(Cursor::new(&mut data[..])).unwrap();
}

#[test]
fn check_mbr_boot_indicator() {
    let mut data = [0u8; 512];
    data[510..].copy_from_slice(&[0x55, 0xAA]);

    for i in 0..4usize {
        data[446 + (i.saturating_sub(1) * 16)] = 0;
        data[446 + (i * 16)] = 0xFF;
        let e = MasterBootRecord::from(Cursor::new(&mut data[..])).unwrap_err();
        expect_variant!(e, ::mbr::Error::UnknownBootIndicator(p) if p == i as u8);
    }

    data[446 + (3 * 16)] = 0;
    MasterBootRecord::from(Cursor::new(&mut data[..])).unwrap();
}

#[test]
#[ignore]
fn test_mbr() {
    let mut mbr = resource!("mbr.img");
    let mut data = [0u8; 512];
    mbr.read_exact(&mut data).expect("read resource data");
    MasterBootRecord::from(Cursor::new(&mut data[..])).expect("valid MBR");
}

#[test]
fn check_ebpb_size() {
    check_size!(BiosParameterBlock, 512);
}

#[test]
fn check_ebpb_signature() {
    let mut data = [0u8; 1024];
    data[510..512].copy_from_slice(&[0x55, 0xAA]);

    let e = BiosParameterBlock::from(Cursor::new(&mut data[..]), 1).unwrap_err();
    expect_variant!(e, ::vfat::Error::BadSignature);

    BiosParameterBlock::from(Cursor::new(&mut data[..]), 0).unwrap();
}

#[test]
#[ignore]
fn test_ebpb() {
    let mut ebpb1 = resource!("ebpb1.img");
    let mut ebpb2 = resource!("ebpb2.img");

    let mut data = [0u8; 1024];
    ebpb1.read_exact(&mut data[..512]).expect("read resource data");
    ebpb2.read_exact(&mut data[512..]).expect("read resource data");

    BiosParameterBlock::from(Cursor::new(&mut data[..]), 0).expect("valid EBPB");
    BiosParameterBlock::from(Cursor::new(&mut data[..]), 1).expect("valid EBPB");
}

#[test]
fn check_entry_sizes() {
    check_size!(::vfat::dir::VFatRegularDirEntry, 32);
    check_size!(::vfat::dir::VFatUnknownDirEntry, 32);
    check_size!(::vfat::dir::VFatLfnDirEntry, 32);
    check_size!(::vfat::dir::VFatDirEntry, 32);
}

#[test]
#[ignore]
fn test_vfat_init() {
    vfat_from_resource!("mock1.fat32.img");
    vfat_from_resource!("mock2.fat32.img");
    vfat_from_resource!("mock3.fat32.img");
}

fn hash_entry<T: Entry>(hash: &mut String, entry: &T) -> ::std::fmt::Result {
    use std::fmt::Write;

    fn write_bool(to: &mut String, b: bool, c: char) -> ::std::fmt::Result {
        if b { write!(to, "{}", c) } else { write!(to, "-") }
    }

    fn write_timestamp<T: Timestamp>(to: &mut String, ts: T) -> ::std::fmt::Result {
        write!(to, "{:02}/{:02}/{} {:02}:{:02}:{:02} ",
               ts.month(), ts.day(), ts.year(), ts.hour(), ts.minute(), ts.second())
    }

    write_bool(hash, entry.is_dir(), 'd')?;
    write_bool(hash, entry.is_file(), 'f')?;
    write_bool(hash, entry.metadata().read_only(), 'r')?;
    write_bool(hash, entry.metadata().hidden(), 'h')?;
    write!(hash, "\t")?;

    write_timestamp(hash, entry.metadata().created())?;
    write_timestamp(hash, entry.metadata().modified())?;
    write_timestamp(hash, entry.metadata().accessed())?;
    write!(hash, "\t")?;

    write!(hash, "{}", entry.name())?;
    Ok(())
}

fn hash_dir<T: Dir>(
    hash: &mut String, dir: T
) -> Result<Vec<T::Entry>, ::std::fmt::Error> {
    let mut entries: Vec<_> = dir.entries()
        .expect("entries interator")
        .collect();

    entries.sort_by(|a, b| a.name().cmp(b.name()));
    for (i, entry) in entries.iter().enumerate() {
        if i != 0 { hash.push('\n'); }
        hash_entry(hash, entry)?;
    }

    Ok(entries)
}

fn hash_dir_from<P: AsRef<Path>>(vfat: Shared<VFat>, path: P) -> String {
    let mut hash = String::new();
    hash_dir(&mut hash, vfat.open_dir(path).expect("directory exists")).unwrap();
    hash
}

#[test]
#[ignore]
fn test_root_entries() {
    let hash = hash_dir_from(vfat_from_resource!("mock1.fat32.img"), "/");
    assert_hash_eq!("mock 1 root directory", hash, hash_for!("root-entries-1"));

    let hash = hash_dir_from(vfat_from_resource!("mock2.fat32.img"), "/");
    assert_hash_eq!("mock 2 root directory", hash, hash_for!("root-entries-2"));

    let hash = hash_dir_from(vfat_from_resource!("mock3.fat32.img"), "/");
    assert_hash_eq!("mock 3 root directory", hash, hash_for!("root-entries-3"));
}

fn hash_dir_recursive<P: AsRef<Path>>(
    hash: &mut String,
    vfat: Shared<VFat>,
    path: P
) -> ::std::fmt::Result {
    use std::fmt::Write;

    let path = path.as_ref();
    let dir = vfat.open_dir(path).expect("directory");

    write!(hash, "{}\n", path.display())?;
    let entries = hash_dir(hash, dir)?;
    if entries.iter().any(|e| e.is_dir()) {
        hash.push_str("\n\n");
    }

    for entry in entries {
        if entry.is_dir() && entry.name() != "." && entry.name() != ".." {
            let path = path.join(entry.name());
            hash_dir_recursive(hash, vfat.clone(), path)?;
        }
    }

    Ok(())
}

fn hash_dir_recursive_from<P: AsRef<Path>>(vfat: Shared<VFat>, path: P) -> String {
    let mut hash = String::new();
    hash_dir_recursive(&mut hash, vfat, path).unwrap();
    hash
}

#[test]
#[ignore]
fn test_all_dir_entries() {
    let hash = hash_dir_recursive_from(vfat_from_resource!("mock1.fat32.img"), "/");
    assert_hash_eq!("mock 1 all dir entries", hash, hash_for!("all-entries-1"));

    let hash = hash_dir_recursive_from(vfat_from_resource!("mock2.fat32.img"), "/");
    assert_hash_eq!("mock 2 all dir entries", hash, hash_for!("all-entries-2"));

    let hash = hash_dir_recursive_from(vfat_from_resource!("mock3.fat32.img"), "/");
    assert_hash_eq!("mock 3 all dir entries", hash, hash_for!("all-entries-3"));
}

fn hash_file<T: File>(hash: &mut String, mut file: T) -> ::std::fmt::Result {
    use std::fmt::Write;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;

    let mut hasher = DefaultHasher::new();
    loop {
        let mut buffer = [0; 4096];
        match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => hasher.write(&buffer[..n]),
            Err(e) => panic!("failed to read file: {:?}", e)
        }
    }

    write!(hash, "{}", hasher.finish())
}

fn hash_files_recursive<P: AsRef<Path>>(
    hash: &mut String,
    vfat: Shared<VFat>,
    path: P
) -> ::std::fmt::Result {
    let path = path.as_ref();
    let mut entries = vfat.open_dir(path)
        .expect("directory")
        .entries()
        .expect("entries interator")
        .collect::<Vec<_>>();

    entries.sort_by(|a, b| a.name().cmp(b.name()));
    for entry in entries {
        let path = path.join(entry.name());
        if entry.is_file() && !entry.name().starts_with(".BC.T") {
            use std::fmt::Write;
            let file = entry.into_file().unwrap();
            if file.size() < (1 << 20) {
                write!(hash, "{}: ", path.display())?;
                hash_file(hash, file).expect("successful hash");
                hash.push('\n');
            }
        } else if entry.is_dir() && entry.name() != "." && entry.name() != ".." {
            hash_files_recursive(hash, vfat.clone(), path)?;
        }
    }

    Ok(())
}

fn hash_files_recursive_from<P: AsRef<Path>>(vfat: Shared<VFat>, path: P) -> String {
    let mut hash = String::new();
    hash_files_recursive(&mut hash, vfat, path).unwrap();
    hash
}

#[test]
#[ignore]
fn test_mock1_files_recursive() {
    let hash = hash_files_recursive_from(vfat_from_resource!("mock1.fat32.img"), "/");
    assert_hash_eq!("mock 1 file hashes", hash, hash_for!("files-1"));
}

#[test]
#[ignore]
fn test_mock2_files_recursive() {
    let hash = hash_files_recursive_from(vfat_from_resource!("mock2.fat32.img"), "/");
    assert_hash_eq!("mock 2 file hashes", hash, hash_for!("files-2-3"));
}

#[test]
#[ignore]
fn test_mock3_files_recursive() {
    let hash = hash_files_recursive_from(vfat_from_resource!("mock3.fat32.img"), "/");
    assert_hash_eq!("mock 3 file hashes", hash, hash_for!("files-2-3"));
}

#[test]
fn shared_fs_is_sync_send_static() {
    fn f<T: Sync + Send + 'static>() {  }
    f::<Shared<VFat>>();
}

#[test]
fn read_vfat() {
    use std::fs::File;
    let mut device = File::open("../fs.img").unwrap();
    let vfat = VFat::from(device).unwrap();
    println!("{:#?}", vfat);

    if false {
        let mut buf = vec![0; 512];
        let mut vfat = vfat.borrow_mut();
        vfat.read_root_dir_cluster(&mut buf[..]);
        println!("{:?}", buf);
    }

    use traits::*;

    let root = ::vfat::dir::Dir::root(vfat.clone());

    println!("FUCK THIS SHIT:");
    for e in root.entries().unwrap() {
        if let Some(e) = e.as_file() {
            println!("FILE: {} {}", e.name, e.meta);
        }
        if let Some(e) = e.as_dir() {
            println!(" DIR: {} {}", e.name, e.meta);
        }
    }
}
