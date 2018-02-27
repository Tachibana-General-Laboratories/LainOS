use std::io::Cursor;

use vfat::*;
use mbr::*;

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
fn check_entry_sizes() {
    check_size!(::vfat::dir::VFatRegularDirEntry, 32);
    check_size!(::vfat::dir::VFatUnknownDirEntry, 32);
    check_size!(::vfat::dir::VFatLfnDirEntry, 32);
    check_size!(::vfat::dir::VFatDirEntry, 32);
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
    let mut buf = vec![0; 512];
    let mut vfat = vfat.borrow_mut();
    vfat.read_root_dir_cluster(&mut buf[..]);

    println!("{:?}", buf);

    use vfat::dir::*;
    for e in buf.chunks(32) {
        let mut chunk = [0u8; 32];
        chunk[..].copy_from_slice(e);

        let e: VFatRegularDirEntry = unsafe { ::std::mem::transmute(chunk) };
        let filter: &[_] = &['\0', ' '];
        let name = ::std::str::from_utf8(&e.name[..]).unwrap().trim_right_matches(filter);
        let ext = ::std::str::from_utf8(&e.ext[..]).unwrap().trim_right_matches(filter);
        println!("{}.{}:::{:?}", name, ext, e);
    }

    let mut file = Vec::new();
    vfat.read_chain(Cluster::from(3), &mut file).unwrap();
    let file = ::std::str::from_utf8(&file).unwrap().trim_right_matches('\0');
    println!("{}", file);

    // readdir
    let mut dir = Vec::new();
    let cluster = vfat.root_dir_cluster;
    vfat.read_chain(cluster, &mut dir).unwrap();

    let entries: &[VFatDirEntry] = unsafe {
        let p = dir.as_ptr() as *const VFatDirEntry;
        ::std::slice::from_raw_parts(p, dir.len() / 4)
    };

    for name in DirIter::new(entries) {
        println!("{}", name);
    }
}
