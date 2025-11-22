use serde::Deserialize;
use serde::de::Deserializer;
use std::collections::HashMap;

// Lets the user supply Janet bools and numbers for things like ZFS and ipadm properties
pub fn value_to_string(v: serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s,
        serde_json::Value::Bool(b) => if b { "on" } else { "off" }.to_owned(),
        _ => v.to_string(),
    }
}

// Deserializes option properties
pub fn option_property_deserializer<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = HashMap::<String, serde_json::Value>::deserialize(deserializer)?;
    let converted = raw
        .into_iter()
        .map(|(k, v)| (k, value_to_string(v)))
        .collect();

    Ok(Some(converted))
}

// Deserializes HashMap properties
pub fn hash_property_deserializer<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, HashMap<String, String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = HashMap::<String, HashMap<String, serde_json::Value>>::deserialize(deserializer)?;
    let converted = raw
        .into_iter()
        .map(|(proto, props)| {
            let converted_props = props
                .into_iter()
                .map(|(k, v)| (k, value_to_string(v)))
                .collect();
            (proto, converted_props)
        })
        .collect();
    Ok(converted)
}
