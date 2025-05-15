use crate::doers::directory::DirectoryResource;

#[derive(Debug)]
pub enum Resource {
    Directory(DirectoryResource),
}

impl Resource {
    fn apply(&self) {
        match self {
            Resource::Directory(d) => d.apply(),
        }
    }
}
