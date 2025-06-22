use crate::common::types::{Action, ApplySummary};
use anyhow::{Context, bail};
use camino::Utf8PathBuf;
use janetrs::{Janet, JanetArray, JanetStruct, JanetTuple, TaggedJanet, client::JanetClient};
use std::fmt::Write;

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

pub fn janet_client() -> JanetClient {
    tracing::debug!("Initialising janet client");
    JanetClient::init_with_default_env().expect("Failed to create Janet client")
}

pub trait JanetExt {
    fn extract_array(&self) -> anyhow::Result<JanetArray>;
    fn extract_tuple(&self) -> anyhow::Result<JanetTuple>;
    fn extract_struct(&self) -> anyhow::Result<JanetStruct>;
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
    fn get_field_bool(&self, field: &str) -> anyhow::Result<bool>;
    fn get_field_u32_string_key(&self, field: &str) -> anyhow::Result<u32>;
    fn get_field_string_tuple(&self, field: &str) -> anyhow::Result<Vec<String>>;
    fn get_field_struct(&self, field: &str) -> anyhow::Result<JanetStruct>;
    fn get_field_struct_opt(&self, field: &str) -> Option<JanetStruct>;
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

    fn get_field_struct(&self, field: &str) -> anyhow::Result<JanetStruct> {
        match self.get_field(field)?.unwrap() {
            TaggedJanet::Struct(s) => Ok(s),
            other => bail!("Expected struct, found {}", other),
        }
    }

    fn get_field_struct_opt(&self, field: &str) -> Option<JanetStruct> {
        match self.get(Janet::keyword(field.into())) {
            Some(j) => match j.unwrap() {
                TaggedJanet::Struct(s) => Some(s),
                _ => None,
            },
            None => None,
        }
    }

    fn get_field_bool(&self, field: &str) -> anyhow::Result<bool> {
        let value = self.get_field(field)?;

        if value == Janet::from(true) {
            Ok(true)
        } else if value == Janet::from(false) {
            Ok(false)
        } else {
            bail!("cannot turn {} into a bool", value)
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

pub fn pretty_janet(j: &Janet, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    match j.unwrap() {
        TaggedJanet::Keyword(k) => format!("{}", k),
        TaggedJanet::String(s) => format!("{:?}", s),
        TaggedJanet::Boolean(b) => format!("{}", b),
        TaggedJanet::Number(n) => format!("{}", n),
        TaggedJanet::Tuple(tup) => {
            let elems = tup
                .iter()
                .map(|x| pretty_janet(x, indent + 1))
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}]", elems)
        }
        TaggedJanet::Array(arr) => {
            let elems = arr
                .iter()
                .map(|x| pretty_janet(x, indent + 1))
                .collect::<Vec<_>>()
                .join(", ");
            format!("[{}]", elems)
        }
        TaggedJanet::Struct(s) => {
            let mut out = String::new();
            writeln!(&mut out, "{{").unwrap();
            for (k, v) in s.iter() {
                writeln!(
                    &mut out,
                    "{}  {} {}",
                    pad,
                    pretty_janet(k, 0),
                    pretty_janet(v, indent + 1)
                )
                .unwrap();
            }
            write!(&mut out, "{}}}", pad).unwrap();
            out
        }
        _ => format!("{:?}", j),
    }
}

use std::collections::HashMap;

// Very crudely convert a Janet Struct into a HashMap. Keys must by strings or symbols, values
// must be scalar. Anything not meeting these rules is silently passed over.
pub fn struct_to_hash(j_struct: &JanetStruct) -> HashMap<String, String> {
    let mut ret = HashMap::new();

    for (k, v) in j_struct {
        let hash_key = match k.unwrap() {
            TaggedJanet::Keyword(k) => k.to_string().trim_start_matches(':').to_owned(),
            TaggedJanet::String(k) => k.to_string(),
            _ => continue,
        };

        let hash_value = match v.unwrap() {
            TaggedJanet::Keyword(v) => v.to_string(),
            TaggedJanet::String(v) => v.to_string(),
            TaggedJanet::Number(v) => v.to_string(),
            TaggedJanet::Boolean(v) => v.to_string(),
            _ => continue,
        };

        ret.insert(hash_key, hash_value);
    }

    ret
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utils::spec_helper::init_janet;
    use janetrs::JanetKeyword;
    use janetrs::array;
    use janetrs::structs;
    use pretty_assertions::assert_eq;

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

    #[test]
    fn test_struct_to_hash() {
        init_janet();
        let arr = array![1, 2, 3];

        let j_struct = structs! {
            ":key-1" => "string-val-1",
            ":key-2" => 2,
            ":key-3" => ":keyword-val-3",
            ":key-4" => arr,
            ":key-5" => true,
        };

        assert_eq!(
            HashMap::from([
                ("key-1".to_owned(), "string-val-1".to_owned()),
                ("key-2".to_owned(), "2".to_owned()),
                ("key-3".to_owned(), ":keyword-val-3".to_owned()),
                ("key-5".to_owned(), "true".to_owned()),
            ]),
            struct_to_hash(&j_struct)
        );
    }
}
