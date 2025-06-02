use crate::debug;
use crate::doers::types::ApplySummary;
use crate::utils::types::Opts;
use anyhow::{Context, anyhow};
use camino::Utf8PathBuf;
use janetrs::client::JanetClient;
use janetrs::{Janet, JanetArray, JanetStruct, TaggedJanet};

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
            .get_field_u32("resources")
            .context("no resources in apply summary")?,
        changes: apply_struct
            .get_field_u32("changes")
            .context("no changes in apply summary")?,
        errors: apply_struct
            .get_field_u32("errors")
            .context("no errors in apply summary")?,
    })
}

pub fn janet_client(opts: &Opts) -> JanetClient {
    debug!(opts, "Initialising janet client");
    JanetClient::init_with_default_env().expect("Failed to create Janet client")
}

pub trait JanetExt {
    fn extract_struct(&self) -> anyhow::Result<JanetStruct>;
    fn extract_array(&self) -> anyhow::Result<JanetArray>;
}

impl JanetExt for Janet {
    fn extract_struct(&self) -> anyhow::Result<JanetStruct> {
        let extracted = self.unwrap();

        let data = match extracted {
            TaggedJanet::Struct(data) => data,
            _ => {
                return Err(anyhow!(format!("did not find struct in {:?}", self)));
            }
        };

        Ok(data)
    }

    fn extract_array(&self) -> anyhow::Result<JanetArray> {
        let extracted = self.unwrap();

        let array = match extracted {
            TaggedJanet::Array(array) => array,
            _ => {
                return Err(anyhow!(format!("did not find array in {:?}", self)));
            }
        };

        Ok(array)
    }
}

pub trait JanetStructExt {
    fn get_field_string(&self, field: &str) -> anyhow::Result<String>;
    // fn get_field_bool(&self, field: &str) -> anyhow::Result<bool>;
    fn get_field_pathbuf(&self, field: &str) -> anyhow::Result<Utf8PathBuf>;
    fn get_field_u32(&self, field: &str) -> anyhow::Result<u32>;
    fn get_field_string_array(&self, field: &str) -> anyhow::Result<Vec<String>>;
}

impl JanetStructExt for JanetStruct<'_> {
    fn get_field_u32(&self, field: &str) -> anyhow::Result<u32> {
        let j_val = self
            .get(field)
            .context(format!(
                "no '{}' field in {:?}",
                Janet::keyword(field.into()),
                self
            ))?
            .unwrap();

        match j_val {
            TaggedJanet::Number(n) => Ok(n as u32),
            other => Err(anyhow!("Expected number, found {}", other)),
        }
    }

    fn get_field_string(&self, field: &str) -> anyhow::Result<String> {
        let ret = self
            .get(Janet::keyword(field.into()))
            .context(format!(
                "no '{}' field in {:?}",
                Janet::keyword(field.into()),
                self
            ))?
            .to_string();

        Ok(ret)
    }

    /*
    fn get_field_bool(&self, field: &str) -> anyhow::Result<bool> {
        let value = self
            .get(Janet::keyword(field.into()))
            .context(format!("directory has no {}", field))?;

        if value == Janet::from(true) {
            Ok(true)
        } else if value == Janet::from(false) {
            Ok(false)
        } else {
            Err(anyhow!(format!("Cannot turn {} into bool", value)))
        }
    }
    */

    fn get_field_pathbuf(&self, field: &str) -> anyhow::Result<Utf8PathBuf> {
        let path = Utf8PathBuf::from(
            self.get(Janet::keyword(field.into()))
                .context(format!(
                    "no '{}' field in {:?}",
                    Janet::keyword(field.into()),
                    self
                ))?
                .to_string(),
        );

        // Relative paths will never be okay. Relative to what?
        if path.is_relative() {
            return Err(anyhow!("Path is relative: {}", path));
        }

        Ok(path)
    }

    fn get_field_string_array(&self, field: &str) -> anyhow::Result<Vec<String>> {
        use crate::utils::janet_helpers::JanetExt;

        let ret = self
            .get(Janet::keyword(field.into()))
            .context(format!(
                "no '{}' field in {:?}",
                Janet::keyword(field.into()),
                self
            ))?
            .extract_array()?
            .iter()
            .filter_map(|item| match item.unwrap() {
                TaggedJanet::String(val) => Some(val.to_string()),
                _ => None,
            })
            .collect();

        Ok(ret)
    }
}

#[cfg(test)]
mod test {
    use crate::test_utils::spec_helper::init_janet;

    use super::*;

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
}
