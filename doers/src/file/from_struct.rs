use crate::file::actions;
use crate::file::types::{CompareMethod, DesiredFileState, OutputFileFormat};
use anyhow::{Context, bail};
use camino::Utf8Path;
use common::types::{ApplyOpts, ApplySummary};
use serde_json::Value;

pub fn run(
    path: &Utf8Path,
    desired_state: &DesiredFileState,
    compare: &CompareMethod,
    opts: &ApplyOpts,
) -> anyhow::Result<ApplySummary> {
    let user_struct = desired_state
        .from_struct
        .as_ref()
        .context("no user struct")?;
    let new_content = to_file(&user_struct, desired_state.to_format.as_ref())?;

    Ok(ApplySummary {
        resources: 1,
        changes: actions::ensure_content(path, &new_content, desired_state, compare, opts)?
            + actions::ensure_metadata(path, desired_state, opts)?,
    })
}

fn to_file(user_struct: &Value, to_format: Option<&OutputFileFormat>) -> anyhow::Result<String> {
    if let Some(format) = to_format {
        match format {
            OutputFileFormat::Yaml => Ok(serde_yaml_bw::to_string(&user_struct)?),
            OutputFileFormat::Toml => Ok(toml::to_string(&user_struct)?),
            OutputFileFormat::Json => Ok(serde_json::to_string_pretty(&user_struct)?),
            OutputFileFormat::Ini => Ok(to_ini(user_struct)?),
            OutputFileFormat::KeyValue => Ok(to_kv(user_struct)?),
        }
    } else {
        bail!("from_struct requires to_format")
    }
}

// Ini files can't nest structs. If we get anything we don't expect, error. This is very basic.
fn to_ini(user_struct: &Value) -> anyhow::Result<String> {
    let map = user_struct
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Requested INI, but data is not a struct"))?;

    let mut ret = String::new();

    for (section_name, values) in map {
        let section_map = values
            .as_object()
            .ok_or_else(|| anyhow::anyhow!("Section '{}' must be a struct", section_name))?;

        if !ret.is_empty() {
            ret.push('\n');
        }

        ret.push_str(&format!("[{section_name}]\n"));

        for (k, v) in section_map {
            let string_k = prepped_kvp(k);
            let string_v = prepped_kvp(&v.to_string());

            let value = if string_v.chars().all(|c| c.is_alphanumeric()) {
                string_v
            } else {
                format!("\"{string_v}\"")
            };

            ret.push_str(&format!("{string_k} = {value}\n"));
        }
    }

    Ok(ret)
}

// Very crude key-value pair. Accepts a map, or an array where alternate entries are key then
// value. The latter lets you have duplicate keys, which I need.
fn to_kv(user_struct: &Value) -> anyhow::Result<String> {
    let mut ret = String::new();

    if let Some(map) = user_struct.as_object() {
        for (k, v) in map {
            let clean_val = v.to_string().trim_matches('"').to_owned();
            ret.push_str(&format!("{k}={clean_val}\n"));
        }
    } else if let Some(map) = user_struct.as_array() {
        if map.len() % 2 != 0 {
            bail!(
                "KVP array must have an even number of elements. (Got {})",
                map.len()
            );
        }

        for chunk in map.chunks(2) {
            let string_k = prepped_kvp(&chunk[0].to_string());
            let string_v = prepped_kvp(&chunk[1].to_string());
            ret.push_str(&format!("{string_k}={string_v}\n"));
        }
    } else {
        bail!("Requested k=v, but data is not a struct or array")
    }

    Ok(ret)
}

fn prepped_kvp(raw: &str) -> String {
    raw.to_string().trim_matches(['"', ':']).to_owned()
}

#[cfg(test)]
mod test {
    use crate::file::ensure::GurpFileEnsure;
    use camino::Utf8PathBuf;
    use camino_tempfile_ext::prelude::*;
    use common::constants::ONE_RESOURCE_ONE_CHANGE;
    use indoc::{formatdoc, indoc};
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tester::{defopts, janet2json, my_group, my_user};

    #[test]
    fn test_file_create_json_from_struct() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test-file");

        assert!(!temp_file.exists());

        let expected = indoc! { r#"
                {
                  "my-arr": [
                    "abc",
                    "def",
                    "ghi"
                  ],
                  "my-str": "I am a String",
                  "my-struct": {
                    "key_1": "val 1",
                    "key_2": 123,
                    "key_3": [
                      456,
                      789
                    ]
                  }
                }"#};

        let sut: GurpFileEnsure = serde_json::from_str(&sample_struct(&temp_file, "json")).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(temp_file.exists());
        let metadata = fs::metadata(&temp_file).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o600);
        assert_eq!(expected, fs::read_to_string(temp_file).unwrap());
    }

    #[test]
    fn test_file_create_yaml_from_struct() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test-file");

        assert!(!temp_file.exists());

        let expected = indoc! { r#"
            my-arr:
            - abc
            - def
            - ghi
            my-str: I am a String
            my-struct:
              key_1: val 1
              key_2: 123
              key_3:
              - 456
              - 789
          "#};

        let sut: GurpFileEnsure = serde_json::from_str(&sample_struct(&temp_file, "yaml")).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(temp_file.exists());
        let metadata = fs::metadata(&temp_file).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o600);
        assert_eq!(expected, fs::read_to_string(temp_file).unwrap());
    }

    #[test]
    fn test_file_create_ini_from_struct() {
        let temp_dir = Utf8TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test-file");

        assert!(!temp_file.exists());

        let json_def = janet2json(&formatdoc! {r#"
            (file/ensure "{path}"
                :from-struct {{
                    :section_1 {{
                        :key_1 "A spacey string"
                        :key_2 123
                        :key_3 false
                        :key_4 "word"
                    }}
                    :section_2 {{
                        :key_1 "merp"
                        :key_2 "gurp"
                    }}
                }}
                :to-format "ini"
                :mode "0600"
                :owner "{user}"
                :group "{group}")
            "#,
            path = temp_file,
            user = my_user(),
            group = my_group(),
        });

        let expected = indoc! { r#"
                [section_1]
                key_1 = "A spacey string"
                key_2 = 123
                key_3 = false
                key_4 = word

                [section_2]
                key_1 = merp
                key_2 = gurp
        "#};

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(temp_file.exists());
        let metadata = fs::metadata(&temp_file).unwrap();
        assert_eq!(metadata.permissions().mode() & 0o7777, 0o600);
        assert_eq!(expected, fs::read_to_string(temp_file).unwrap());
    }

    #[test]
    fn test_file_create_ini_from_struct_errors() {
        let sut: GurpFileEnsure =
            serde_json::from_str(&sample_struct(&Utf8PathBuf::from("/tmp/file"), "ini")).unwrap();
        assert!(sut.apply(&defopts()).is_err());
    }

    fn sample_struct(path: &Utf8PathBuf, format: &str) -> String {
        janet2json(&formatdoc! {r#"
            (file/ensure "{path}"
                :from-struct {{
                    :my-struct {{
                        :key_1 "val 1"
                        :key_2 123
                        :key_3 [456 789]
                    }}
                    :my-arr ["abc" "def" "ghi"]
                    :my-str "I am a String"
                }}
                :to-format "{format}"
                :mode "0600"
                :owner "{user}"
                :group "{group}")
            "#,
            path = path,
            format = format,
            user = my_user(),
            group = my_group(),
        })
    }
}
