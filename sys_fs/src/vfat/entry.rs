use sys;
use vfat::{File, Dir, Metadata, Cluster};

// TODO: You may need to change this definition.
#[derive(Debug)]
pub enum Entry {
    File(File),
    Dir(Dir)
}

// TODO: Implement any useful helper methods on `Entry`.
impl Entry {
    fn cluster(&self) -> Cluster {
        match self {
            &Entry::File(ref e) => e.cluster,
            &Entry::Dir(ref e) => e.cluster,
        }
    }

    fn size(&self) -> u64 {
        match self {
            &Entry::File(ref e) => e.size,
            &Entry::Dir(ref e) => e.size,
        }
    }
}

impl sys::fs::Entry for Entry {
    type File = File;
    type Dir = Dir;
    type Metadata = Metadata;

    fn name(&self) -> &str {
        match self {
            &Entry::File(ref e) => &e.name,
            &Entry::Dir(ref e) => &e.name,
        }
    }
    fn metadata(&self) -> &Self::Metadata {
        match self {
            &Entry::File(ref e) => &e.meta,
            &Entry::Dir(ref e) => &e.meta,
        }
    }
    fn as_file(&self) -> Option<&Self::File> {
        match self {
            &Entry::File(ref e) => Some(e),
            _ => None,
        }
    }
    fn as_dir(&self) -> Option<&Self::Dir> {
        match self {
            &Entry::Dir(ref e) => Some(e),
            _ => None,
        }
    }
    fn into_file(self) -> Option<Self::File> {
        match self {
            Entry::File(e) => Some(e),
            _ => None,
        }
    }
    fn into_dir(self) -> Option<Self::Dir> {
        match self {
            Entry::Dir(e) => Some(e),
            _ => None,
        }
    }
}
