use crate::utils::types::VarMap;
use anyhow::{Context, anyhow};
use janetrs::{Janet, JanetTable, TaggedJanet};
use std::collections::HashMap;

// macro_rules! from_janet_table {
//     ($table:expr, $($field:ident : $ty:ty),+ $(,)?) => {
//         (
//             $(
//                 $table.get(stringify!($field)).try_into()
//                     .map_err(|e| anyhow!("Failed to get {}: {}", stringify!($field), e))?
//             ),+
//         )
//     };
// }

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
