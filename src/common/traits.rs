use crate::common::types::{ApplySummary, Opts, Resource};

pub trait Apply {
    fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary>;
}

impl Apply for Resource {
    fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        match self {
            Resource::Directory(inner) => inner.apply(opts),
            Resource::User(inner) => inner.apply(opts),
            Resource::Pkg(inner) => inner.apply(opts),
        }
    }
}

pub trait HasId {
    fn id(&self) -> String;
}

impl HasId for Resource {
    fn id(&self) -> String {
        match self {
            Resource::Directory(inner) => inner.id.clone(),
            Resource::User(inner) => inner.id.clone(),
            Resource::Pkg(inner) => inner.id.clone(),
        }
    }
}
