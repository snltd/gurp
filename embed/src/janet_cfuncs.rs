use crate::constants::GURP_LIB_IMAGE;
use crate::convert;
use janetrs::{Janet, JanetString};

#[janetrs::janet_fn(arity(fix(1)))]
pub fn to_json(config: &mut [Janet]) -> Janet {
    let json_string = convert::janet_to_json(&config[0]).to_string();
    Janet::wrap(json_string.as_str())
}

// Janet strings/buffers are binary-safe, so we can dump an image into one
#[janetrs::janet_fn()]
pub fn gurp_library(_arg: &mut [Janet]) -> Janet {
    let lib_as_string = JanetString::new(GURP_LIB_IMAGE);
    Janet::string(lib_as_string)
}
