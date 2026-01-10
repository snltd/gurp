use crate::constants::GURP_LIB_IMAGE;
use crate::helpers as janet_helpers;
use anyhow::bail;
use janetrs::client::JanetClient;
use janetrs::env::CFunOptions;
use janetrs::{Janet, JanetString, TaggedJanet};
use serde_json::{Map, Value};

/// Returns a standard Janet client, with no Gurp library.
pub fn janet_client() -> JanetClient {
    tracing::debug!("Initialising janet client");
    JanetClient::init_with_default_env().expect("Failed to create Janet client")
}

/// Returns a Janet client with the Gurp library in the root environemnt. Also includes
/// (to-json) which turns any suitable Janet object into Json.
pub fn gurp_client() -> anyhow::Result<JanetClient> {
    let mut client = janet_client();
    client.add_c_fn(CFunOptions::new(c"gurp-library", gurp_library_c));
    client.add_c_fn(CFunOptions::new(c"to-json", janet_helpers::to_json_c));

    let janet_instructions =
        r#"(merge-module (fiber/getenv (fiber/root)) (load-image (gurp-library)) "" true)"#;

    tracing::debug!("creating Janet client with Gurp environment");
    client.run(janet_instructions)?;

    Ok(client)
}

/// Converts Janet objects into JSON
pub fn janet_to_json(j: &Janet) -> Value {
    // I'm going to leave the :s at the beginning of the key names for now, because it will
    // make it clear we're talking about user data.
    match j.unwrap() {
        TaggedJanet::Nil => Value::Null,
        TaggedJanet::Boolean(b) => Value::Bool(b),
        TaggedJanet::Number(n) => {
            if n.fract() == 0.0 {
                match serde_json::Number::from_f64(n) {
                    Some(_num) => Value::Number(serde_json::Number::from(n as i64)),
                    None => Value::Null,
                }
            } else {
                match serde_json::Number::from_f64(n) {
                    Some(num) => Value::Number(num),
                    None => Value::Null,
                }
            }
        }

        TaggedJanet::String(s) => Value::String(s.to_string()),
        TaggedJanet::Symbol(s) => Value::String(s.to_string()),
        TaggedJanet::Keyword(k) => Value::String(k.to_string()),
        TaggedJanet::Array(arr) => {
            let vec = arr.iter().map(janet_to_json).collect();
            Value::Array(vec)
        }
        TaggedJanet::Tuple(tup) => {
            let vec = tup.iter().map(janet_to_json).collect();
            Value::Array(vec)
        }
        TaggedJanet::Table(tab) => {
            let mut map = Map::new();
            for (k, v) in tab.iter() {
                let key = k.to_string().trim_start_matches(':').to_string();
                map.insert(key, janet_to_json(v));
            }
            Value::Object(map)
        }
        TaggedJanet::Struct(tab) => {
            let mut map = Map::new();
            for (k, v) in tab.iter() {
                let key = k.to_string().trim_start_matches(':').to_string();
                map.insert(key, janet_to_json(v));
            }
            Value::Object(map)
        }
        // I don't think we'll need any more exotic types
        other => Value::String(format!("<{other:?}>")),
    }
}

#[janetrs::janet_fn(arity(fix(1)))]
pub fn to_json(config: &mut [Janet]) -> Janet {
    let json_string = janet_to_json(&config[0]).to_string();
    Janet::wrap(json_string.as_str())
}

// Janet strings/buffers are binary-safe, so we can dump an image into one
#[janetrs::janet_fn()]
fn gurp_library(_arg: &mut [Janet]) -> Janet {
    let lib_as_string = JanetString::new(GURP_LIB_IMAGE);
    Janet::string(lib_as_string)
}

pub fn run_config(host_config: &str) -> anyhow::Result<String> {
    let mut client = janet_helpers::janet_client();
    client.add_c_fn(CFunOptions::new(c"to_json", janet_helpers::to_json_c));
    let json_wrapped_host_config = format!("{host_config}\n(to-json (machine-config))");
    let json_config = client.run(json_wrapped_host_config)?;

    let json = match json_config.unwrap() {
        TaggedJanet::String(buf) => buf.to_string(),
        other => bail!("expected JSON config as Janet::String; got {}", other),
    };

    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;
    // use crate::tester::fixture;
    use janetrs::{Janet, JanetTable, array};
    use serde_json::json;

    #[test]
    fn test_janet_client() {
        let client = janet_client();
        assert_eq!(3, janet_to_json(&client.run("(+ 1 2)").unwrap()));
    }

    #[test]
    fn test_gurp_client() {
        let client = gurp_client().unwrap();
        assert_eq!(3, janet_to_json(&client.run("(+ 1 2)").unwrap()));

        assert_eq!(
            "/path/to/file",
            janet_to_json(&client.run(r#"(pathcat "path" "to" "file")"#).unwrap())
        );

        assert_eq!(
            r#"{"a":123}"#,
            janet_to_json(&client.run(r#"(to-json {:a 123})"#).unwrap())
        );
    }

    #[test]
    fn test_janet_to_json_string() {
        init_janet();
        assert_eq!(janet_to_json(&Janet::from("merp merp")), json!("merp merp"));
    }

    #[test]
    fn test_janet_to_json_number() {
        init_janet();
        assert_eq!(janet_to_json(&Janet::from(12.3)), json!(12.3));
    }

    #[test]
    fn test_janet_to_json_array() {
        init_janet();
        let arr = Janet::wrap(array![
            Janet::from(1),
            Janet::from("two"),
            Janet::from(false)
        ]);
        assert_eq!(janet_to_json(&arr), json!([1, "two", false]));
    }

    #[test]
    fn test_janet_to_json_table() {
        init_janet();
        let mut table = JanetTable::new();
        table.insert(Janet::keyword("number".into()), Janet::from(12.3));
        table.insert(Janet::keyword("word".into()), Janet::from("grease"));
        assert_eq!(
            janet_to_json(&Janet::table(table)),
            json!({ "number": 12.3, "word": "grease" })
        );
    }

    #[test]
    fn test_janet_to_json_nested_table() {
        init_janet();
        let mut inner = JanetTable::new();
        inner.insert(Janet::keyword("x".into()), Janet::from(1.1));
        inner.insert(Janet::keyword("y".into()), Janet::from(2));

        let mut outer = JanetTable::new();
        outer.insert(Janet::keyword("point".into()), Janet::table(inner));
        outer.insert(Janet::keyword("label".into()), Janet::from("A"));

        assert_eq!(
            janet_to_json(&Janet::table(outer)),
            json!({
                "point": { "x": 1.1, "y": 2 },
                "label": "A"
            })
        );
    }

    fn init_janet() {
        unsafe {
            janetrs::lowlevel::janet_init();
        }
    }
}
