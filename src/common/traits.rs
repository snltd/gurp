// use crate::common::types::{ApplyContext, ApplySummary, Opts};

// pub trait Apply {
//     fn apply(&self, apply_context: &ApplyContext, opts: &Opts) -> anyhow::Result<ApplySummary>;
// }

// impl Apply for Resource {
//     fn apply(&self, apply_context: &ApplyContext, opts: &Opts) -> anyhow::Result<ApplySummary> {
//         match self {
//             Resource::Cron(inner) => inner.apply(apply_context, opts),
//             // Resource::Directory(inner) => inner.apply(apply_context, opts),
//             // Resource::File(inner) => inner.apply(apply_context, opts),
//             Resource::FileLine(inner) => inner.apply(apply_context, opts),
//             // Resource::Gem(inner) => inner.apply(apply_context, opts),
//             Resource::Misc(inner) => inner.apply(apply_context, opts),
//             // Resource::Pkg(inner) => inner.apply(apply_context, opts),
//             Resource::Smf(inner) => inner.apply(apply_context, opts),
//             Resource::Svc(inner) => inner.apply(apply_context, opts),
//             Resource::Symlink(inner) => inner.apply(apply_context, opts),
//             Resource::User(inner) => inner.apply(apply_context, opts),
//             Resource::Zfs(inner) => inner.apply(apply_context, opts),
//         }
//     }
// }
