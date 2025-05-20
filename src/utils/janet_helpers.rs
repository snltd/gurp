use anyhow::{Context, anyhow};
use camino::Utf8PathBuf;
use janetrs::client::JanetClient;
use janetrs::{Janet, JanetArray, JanetStruct, TaggedJanet};

pub fn janet_client() -> JanetClient {
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
    fn get_field_bool(&self, field: &str) -> anyhow::Result<bool>;
    fn get_field_pathbuf(&self, field: &str) -> anyhow::Result<Utf8PathBuf>;
}

impl JanetStructExt for JanetStruct<'_> {
    fn get_field_string(&self, field: &str) -> anyhow::Result<String> {
        let ret = self
            .get(Janet::keyword(field.into()))
            .context(format!("directory has no {}", field))?
            .to_string();

        Ok(ret)
    }

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

    fn get_field_pathbuf(&self, field: &str) -> anyhow::Result<Utf8PathBuf> {
        let path = Utf8PathBuf::from(
            self.get(Janet::keyword(field.into()))
                .context(format!("directory has no {}", field))?
                .to_string(),
        );

        // Relative paths will never be okay. Relative to what?
        if path.is_relative() {
            return Err(anyhow!("Path is relative: {}", path));
        }

        Ok(path)
    }
}
