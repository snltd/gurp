use crate::common::types::{Action, ApplySummary, Opts};
use crate::debug;
use anyhow::{Context, bail};
use camino::Utf8PathBuf;
use janetrs::{Janet, JanetArray, JanetStruct, JanetTuple, TaggedJanet, client::JanetClient};

// We need to pass an ApplySummary through the Rust->Janet->Rust boundary. These two symmetrical
// functions are far simpler than faffing about with JanetAbstract.
pub fn wrap_summary(summary: &ApplySummary) -> Janet {
    let janet_summary = JanetStruct::builder(3)
        .put("resources", summary.resources as i32)
        .put("changes", summary.changes as i32)
        .put("errors", summary.errors as i32)
        .finalize();

    Janet::wrap(janet_summary)
}

pub fn unwrap_summary(summary: &Janet) -> anyhow::Result<ApplySummary> {
    let apply_struct = summary.extract_struct()?;

    Ok(ApplySummary {
        resources: apply_struct
            .get_field_u32_string_key("resources")
            .context("no 'resources' key in apply summary")?,
        changes: apply_struct
            .get_field_u32_string_key("changes")
            .context("no changes in apply summary")?,
        errors: apply_struct
            .get_field_u32_string_key("errors")
            .context("no errors in apply summary")?,
    })
}

pub fn janet_client(opts: &Opts) -> JanetClient {
    debug!(opts, "janet/client", "Initialising janet client");
    JanetClient::init_with_default_env().expect("Failed to create Janet client")
}

pub trait JanetExt {
    fn extract_struct(&self) -> anyhow::Result<JanetStruct>;
    fn extract_array(&self) -> anyhow::Result<JanetArray>;
    fn extract_tuple(&self) -> anyhow::Result<JanetTuple>;
}

impl JanetExt for Janet {
    fn extract_struct(&self) -> anyhow::Result<JanetStruct> {
        match self.unwrap() {
            TaggedJanet::Struct(data) => Ok(data),
            _ => bail!("did not find struct in {:?}", self),
        }
    }

    fn extract_array(&self) -> anyhow::Result<JanetArray> {
        match self.unwrap() {
            TaggedJanet::Array(data) => Ok(data),
            _ => bail!("did not find array in {:?}", self),
        }
    }

    fn extract_tuple(&self) -> anyhow::Result<JanetTuple> {
        match self.unwrap() {
            TaggedJanet::Tuple(data) => Ok(data),
            _ => bail!("did not find tuple in {:?}", self),
        }
    }
}

pub trait JanetStructExt {
    fn get_field(&self, field: &str) -> anyhow::Result<Janet>;
    fn get_field_string_opt(&self, field: &str) -> Option<String>;
    fn get_field_string(&self, field: &str) -> anyhow::Result<String>;
    fn get_field_pathbuf(&self, field: &str) -> anyhow::Result<Utf8PathBuf>;
    fn get_field_u32(&self, field: &str) -> anyhow::Result<u32>;
    fn get_field_u32_string_key(&self, field: &str) -> anyhow::Result<u32>;
    fn get_field_string_tuple(&self, field: &str) -> anyhow::Result<Vec<String>>;
}

impl JanetStructExt for JanetStruct<'_> {
    fn get_field(&self, field: &str) -> anyhow::Result<Janet> {
        match self.get(Janet::keyword(field.into())) {
            Some(val) => Ok(Janet::wrap(val)),
            None => bail!(
                "no '{}' field in {:?}",
                Janet::keyword(field.into()),
                self.keys()
            ),
        }
    }

    fn get_field_u32(&self, field: &str) -> anyhow::Result<u32> {
        match self.get_field(field)?.unwrap() {
            TaggedJanet::Number(n) => Ok(n as u32),
            other => bail!("Expected number, found {}", other),
        }
    }

    fn get_field_string(&self, field: &str) -> anyhow::Result<String> {
        Ok(self.get_field(field)?.unwrap().to_string())
    }

    fn get_field_string_opt(&self, field: &str) -> Option<String> {
        self.get(Janet::keyword(field.into()))
            .map(|s| s.unwrap().to_string())
    }

    // This is needed in one particular case. We wrap our final ApplySummary in a Janet at
    // the end of the Janet callback, then we unpack it in main.rs. Because the original Janet
    // execution is complete at that point, we need to initialise a second Janet interpreted to
    // unpack the wrapped ApplySummary. Using keywords as keys, as we normally do in Janet, does
    // not work in this circumstance because keywords effectively map to a memory address, so
    // :resources in the first interpreter is not the same as :resources in the second. But,
    // "resources" is the same wherever you are. So we use string keys in that one circumstance.
    //
    //
    fn get_field_u32_string_key(&self, field: &str) -> anyhow::Result<u32> {
        let j_val = self
            .get(field)
            .context(format!(
                "no '{}' field in {:?}",
                Janet::keyword(field.into()),
                self.keys()
            ))?
            .unwrap();

        match j_val {
            TaggedJanet::Number(n) => Ok(n as u32),
            other => bail!("Expected number, found {}", other),
        }
    }

    fn get_field_pathbuf(&self, field: &str) -> anyhow::Result<Utf8PathBuf> {
        let path = Utf8PathBuf::from(self.get_field(field)?.unwrap().to_string());

        // Relative paths will never be okay. Relative to what?
        if path.is_relative() {
            bail!("Path is relative: {}", path);
        }

        Ok(path)
    }

    fn get_field_string_tuple(&self, field: &str) -> anyhow::Result<Vec<String>> {
        use crate::utils::janet_helpers::JanetExt;

        let ret = self
            .get_field(field)?
            .extract_tuple()?
            .iter()
            .filter_map(|item| match item.unwrap() {
                TaggedJanet::String(val) => Some(val.to_string()),
                _ => None,
            })
            .collect();

        Ok(ret)
    }
}

pub fn action_as_enum(janet_data: &JanetStruct) -> anyhow::Result<Action> {
    match janet_data.get_field_string("action")?.as_str() {
        ":ensure" => Ok(Action::Ensure),
        ":remove" => Ok(Action::Remove),
        other => bail!("Action must be :ensure or :remove. Got '{}'", other),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::spec_helper::init_janet;
    use janetrs::JanetKeyword;
    use janetrs::structs;

    #[test]
    fn test_wrap_and_unwrap_summary() {
        init_janet();

        let original_summary = ApplySummary {
            resources: 3,
            changes: 2,
            errors: 1,
        };

        let wrapped_summary = wrap_summary(&original_summary);
        assert_eq!(original_summary, unwrap_summary(&wrapped_summary).unwrap());
    }

    #[test]
    fn test_extract_struct() {
        init_janet();
        let inner = structs! { ":name" => "test_name"};
        let has_struct = Janet::wrap(&inner);
        assert_eq!(inner, has_struct.extract_struct().unwrap());
        assert!(
            Janet::wrap(JanetKeyword::from("not-a-struct"))
                .extract_struct()
                .is_err()
        );
    }
}
