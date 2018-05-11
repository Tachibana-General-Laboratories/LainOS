use traits;
use vfat::{File, Dir, Metadata, Cluster};

// TODO: You may need to change this definition.
#[derive(Debug)]
pub enum Entry {
    File(File),
    Dir(Dir)
}

// TODO: Implement any useful helper methods on `Entry`.

// FIXME: Implement `traits::Entry` for `Entry`.
