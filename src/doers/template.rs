use crate::doers::types::{Apply, ApplySummary, Changes, Ensure, Remove};
use crate::utils::janet_helpers::{JanetExt, JanetStructExt};
use crate::utils::types::Opts;
use crate::{debug, info, verbose};
use janetrs::{Janet, JanetArray};

// How a doer works:
// It must have a ThingToEnsure struct, which parallels the struct we get from a Janet /ensure.
// This is all the user-supplied fields, plus the machine-generated :id.
// It must have a ThingToRemove struct, which parallels the struct we get from Janet /remove. This
// is probably just the id and the thing that must be operated on to remove it, such as a path or
// username.
// It must have a ThingEnsureState struct, which is a subset of ThingToEnsure, only containing the
// fields we need to compare.
// ThingToEnsure and ThingToRemove must both implement TryFrom, which maps the Janet struct to the
// Rust one; and an apply() function. In the Ensure case this
// must be able to create a new Thing, and compare the current state with the desired state,
// aligning them as necessary. In the case of ThingToRemove, it must be able to remove the resource
// (assuming the OS allows it). In both cases, an ApplySummary is returned.
//

#[derive(Debug, PartialEq)]
pub struct ThingToEnsure {
    pub id: String,
}

#[derive(Debug, PartialEq)]
pub struct ThingToRemove {
    pub id: String,
}

#[derive(Debug, PartialEq)]
pub struct ThingEnsureState {}

impl TryFrom<&Janet> for ThingToEnsure {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<ThingToEnsure> {
        let data = value.extract_struct()?;

        Ok(ThingToEnsure {
            id: data.get_field_string("_id")?,
            name: data.get_field_string("name")?,
        })
    }
}

impl TryFrom<&Janet> for ThingToRemove {
    type Error = anyhow::Error;

    fn try_from(value: &Janet) -> anyhow::Result<ThingToRemove> {
        let data = value.extract_struct()?;

        Ok(ThingToRemove {})
    }
}

pub fn unpack_ensure_list(resource_list: &JanetArray) -> anyhow::Result<Vec<Ensure>> {
    resource_list
        .iter()
        .map(|r| {
            let dir = ThingToEnsure::try_from(r)?;
            Ok(Ensure::Thing(dir))
        })
        .collect()
}

pub fn unpack_remove_list(resource_list: &JanetArray) -> anyhow::Result<Vec<Remove>> {
    resource_list
        .iter()
        .map(|r| {
            let dir = ThingToRemove::try_from(r)?;
            Ok(Remove::Thing(dir))
        })
        .collect()
}

fn diff_states<'a>(current: &ThingEnsureState, desired: &ThingEnsureState) -> Changes<'a> {
    let mut to_change = Vec::new();

    if current.whatever != desired.whatever {
        to_change.push("whatever");
    }

    to_change
}

impl ThingToEnsure {
    fn state(&self) -> anyhow::Result<Option<ThingEnsureState>> {
        thing_state(&self.path, &self.name)
    }

    fn desired_state(&self) -> anyhow::Result<ThingEnsureState> {
        Ok(ThingEnsureState {})
    }
}

impl Apply for ThingToEnsure {
    fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        let current_state = if self.path.exists() {
            self.state()
        } else {
            info!(opts, "Creating thing {} [{}]", self.name);

            if opts.noop {
                return Ok(ONE_RESOURCE_ONE_CHANGE);
            }
        }?
        .context(format!("Cannot get state of {}", self.path))?;

        let desired_state = self.desired_state()?;

        let changes = diff_states(&current_state, &desired_state);

        if changes.is_empty() {
            verbose!(
                opts,
                "thing: {} [{}] : no change required",
                self.path,
                self.name
            );
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        // DO CHANGES

        Ok(ONE_RESOURCE_ONE_CHANGE)
    }
}

fn thing_state(path: &Utf8PathBuf, name: &str) -> anyhow::Result<Option<ThingEnsureState>> {
    if path.exists() {
        let metadata = fs::metadata(path)?;
    } else {
        Ok(None)
    }
}

impl Apply for ThingToRemove {
    fn apply(&self, opts: &Opts) -> anyhow::Result<ApplySummary> {
        if !self.path.exists() {
            debug!(
                opts,
                "thing{} [{}]: {} does not exist", self.name, self.id, self.path
            );
            return Ok(ONE_RESOURCE_NO_CHANGE);
        }

        if NOT_ALLOWED_TO_REMOVE.contains(&self.path) {
            eprintln!("Not allowed to remove {}", self.path);
            return Ok(ONE_RESOURCE_ONE_ERROR);
        }

        if opts.noop {
            Ok(ONE_RESOURCE_NOOP)
        } else {
            // REMOVE THING
            Ok(ONE_RESOURCE_ONE_CHANGE)
        }
    }
}
