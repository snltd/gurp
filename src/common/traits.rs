use crate::common::types::{ApplyContext, ApplySummary, Opts, Resource};

pub trait Apply {
    fn apply(&self, apply_context: &ApplyContext, opts: &Opts) -> anyhow::Result<ApplySummary>;
}

impl Apply for Resource {
    fn apply(&self, apply_context: &ApplyContext, opts: &Opts) -> anyhow::Result<ApplySummary> {
        match self {
            Resource::File(inner) => inner.apply(apply_context, opts),
            Resource::Directory(inner) => inner.apply(apply_context, opts),
            Resource::User(inner) => inner.apply(apply_context, opts),
            Resource::Pkg(inner) => inner.apply(apply_context, opts),
            Resource::FileLine(inner) => inner.apply(apply_context, opts),
            Resource::Cron(inner) => inner.apply(apply_context, opts),
            Resource::Svc(inner) => inner.apply(apply_context, opts),
            Resource::Misc(inner) => inner.apply(apply_context, opts),
            Resource::Smf(inner) => inner.apply(apply_context, opts),
        }
    }
}

pub trait HasId {
    fn id(&self) -> String;
}

impl HasId for Resource {
    fn id(&self) -> String {
        match self {
            Resource::File(inner) => inner.id.clone(),
            Resource::Directory(inner) => inner.id.clone(),
            Resource::User(inner) => inner.id.clone(),
            Resource::Pkg(inner) => inner.id.clone(),
            Resource::FileLine(inner) => inner.id.clone(),
            Resource::Cron(inner) => inner.id.clone(),
            Resource::Svc(inner) => inner.id.clone(),
            Resource::Misc(inner) => inner.id.clone(),
            Resource::Smf(inner) => inner.id.clone(),
        }
    }
}
