pub mod sd;

use core::ops::Deref;

pub use sys::fs;
use sys::fs::io;

use fat32::vfat::{self, Shared, VFat};

use sys::Mutex;
use self::sd::Sd;

pub struct FileSystem(pub Mutex<Option<Shared<VFat>>>);

impl FileSystem {
    /// Returns an uninitialized `FileSystem`.
    ///
    /// The file system must be initialized by calling `initialize()` before the
    /// first memory allocation. Failure to do will result in panics.
    pub const fn uninitialized() -> Self {
        FileSystem(Mutex::new(None))
    }

    /// Initializes the file system.
    ///
    /// # Panics
    ///
    /// Panics if the underlying disk or file sytem failed to initialize.
    pub fn initialize(&self) {
        let sd = Sd::new().unwrap();
        let vfat = VFat::from(sd).unwrap();
        *self.0.lock().unwrap() = Some(vfat);
    }
}

impl<'a> fs::FileSystem for &'a FileSystem {
    type File = vfat::File;
    type Dir = vfat::Dir;
    type Entry = vfat::Entry;

    fn open(self, path: &str) -> io::Result<Self::Entry> {
        match self.0.lock().unwrap().deref() {
            &Some(ref vfat) => vfat.open(path),
            &None => panic!("uninitialized"),
        }
    }

    fn create_file(self, path: &str) -> io::Result<Self::File> {
        match self.0.lock().unwrap().deref() {
            &Some(ref vfat) => vfat.create_file(path),
            &None => panic!("uninitialized"),
        }
    }

    fn create_dir(self, path: &str, parents: bool) -> io::Result<Self::Dir> {
        match self.0.lock().unwrap().deref() {
            &Some(ref vfat) => vfat.create_dir(path, parents),
            &None => panic!("uninitialized"),
        }
    }

    fn rename(self, from: &str, to: &str) -> io::Result<()> {
        match self.0.lock().unwrap().deref() {
            &Some(ref vfat) => vfat.rename(from, to),
            &None => panic!("uninitialized"),
        }
    }

    fn remove(self, path: &str, children: bool) -> io::Result<()> {
        match self.0.lock().unwrap().deref() {
            &Some(ref vfat) => vfat.remove(path, children),
            &None => panic!("uninitialized"),
        }
    }
}
