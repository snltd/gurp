use crate::utils::types::VarMap;
use anyhow::{Context, anyhow};
use janetrs::JanetStruct;
use janetrs::{Janet, JanetTable, TaggedJanet};
use serde_json::{Map, Value};
use std::collections::HashMap;

pub fn janet_to_json(j: &Janet) -> Value {
    // I'm going to leave the :s at the beginning of the key names for now, because it will
    // make it clear we're talking about user data.
    match j.unwrap() {
        TaggedJanet::Nil => Value::Null,
        TaggedJanet::Boolean(b) => Value::Bool(b),
        TaggedJanet::Number(n) => match serde_json::Number::from_f64(n) {
            Some(num) => Value::Number(num),
            None => Value::Null,
        },
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
                let key = k.to_string();
                map.insert(key, janet_to_json(v));
            }
            Value::Object(map)
        }
        TaggedJanet::Struct(tab) => {
            let mut map = Map::new();
            for (k, v) in tab.iter() {
                let key = k.to_string();
                map.insert(key, janet_to_json(v));
            }
            Value::Object(map)
        }
        // I don't think we'll need any more exotic types
        other => Value::String(format!("<{:?}>", other)),
    }
}

/*
pub fn unpack_object(janet_object: &Janet) -> anyhow::Result<(String, JanetTable)> {
    let res = match janet_object.unwrap() {
        TaggedJanet::Tuple(res) => res,
        _ => return Err(anyhow!("expected {:?} to be a Janet Tuple", janet_object)),
    };

    let rust_name = res
        .get(0)
        .context(format!(
            "cannot extract name string from {:?}",
            janet_object
        ))?
        .to_string();

    let config = res
        .get(1)
        .context(format!(
            "cannot extract options table from {:?}",
            janet_object
        ))
        .unwrap();

    let config_table = match config.unwrap() {
        TaggedJanet::Table(res) => res,
        _ => return Err(anyhow!("unexpected return type defining host")),
    };

    Ok((rust_name, config_table))
}

pub fn unpack_tuple_of_strings(janet_tuple: &Janet) -> anyhow::Result<Vec<String>> {
    let res = match janet_tuple.unwrap() {
        TaggedJanet::Tuple(res) => res,
        _ => return Err(anyhow!("expected {:?} to be a Janet Tuple", janet_tuple)),
    };

    let elements: Vec<_> = res.iter().map(|t| t.unwrap().to_string()).collect();
    Ok(elements)
}

// I'm not sure about this. It might be better to extract the values as a string and inject it
// into the other user-supplied Janet.
pub fn unpack_var_struct(janet_struct: &Janet) -> anyhow::Result<VarMap> {
    let janet_struct = match janet_struct.unwrap() {
        TaggedJanet::Struct(res) => res,
        _ => return Err(anyhow!("expected {:?} to be a Janet Struct", janet_struct)),
    };

    let ret: HashMap<String, String> = janet_struct
        .iter()
        .map(|(k, v)| (k.to_string().replacen(":", "", 1), v.to_string()))
        .collect();

    Ok(ret)
}
*/
#[cfg(test)]
mod tests {
    use super::*;
    use janetrs::{Janet, JanetTable, array, client::JanetClient};
    use serde_json::{Value, json};

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
            Janet::from(1.0),
            Janet::from("two"),
            Janet::from(false)
        ]);
        assert_eq!(janet_to_json(&arr), json!([1.0, "two", false]));
    }

    #[test]
    fn test_janet_to_json_table() {
        init_janet();
        let mut table = JanetTable::new();
        table.insert(Janet::keyword("number".into()), Janet::from(12.3));
        table.insert(Janet::keyword("word".into()), Janet::from("grease"));
        assert_eq!(
            janet_to_json(&Janet::table(table)),
            json!({ ":number": 12.3, ":word": "grease" })
        );
    }

    #[test]
    fn test_janet_to_json_nested_table() {
        init_janet();
        let mut inner = JanetTable::new();
        inner.insert(Janet::keyword("x".into()), Janet::from(1.0));
        inner.insert(Janet::keyword("y".into()), Janet::from(2.0));

        let mut outer = JanetTable::new();
        outer.insert(Janet::keyword("point".into()), Janet::table(inner));
        outer.insert(Janet::keyword("label".into()), Janet::from("A"));

        assert_eq!(
            janet_to_json(&Janet::table(outer)),
            json!({
                ":point": { ":x": 1.0, ":y": 2.0 },
                ":label": "A"
            })
        );
    }

    fn init_janet() {
        unsafe {
            janetrs::lowlevel::janet_init();
        }
    }
}
