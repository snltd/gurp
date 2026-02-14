use crate::janet_cfuncs;
use janetrs::client::JanetClient;
use janetrs::env::CFunOptions;

/// Returns a standard Janet client, with no Gurp library.
pub fn vanilla() -> JanetClient {
    tracing::debug!("Initialising janet client");
    JanetClient::init_with_default_env().expect("Failed to create Janet client")
}

/// Returns a Janet client with the Gurp library in the root environment. Also includes
/// (to-json) which turns any suitable Janet object into Json.
pub fn gurp() -> anyhow::Result<JanetClient> {
    let mut client = vanilla();
    client.add_c_fn(CFunOptions::new(
        c"gurp-library",
        janet_cfuncs::gurp_library_c,
    ));
    client.add_c_fn(CFunOptions::new(c"to-json", janet_cfuncs::to_json_c));

    let janet_instructions =
        r#"(merge-module (fiber/getenv (fiber/root)) (load-image (gurp-library)) "" true)"#;

    tracing::debug!("creating Janet client with Gurp environment");
    client.run(janet_instructions)?;

    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::convert;

    #[test]
    fn test_vanilla_client() {
        let client = vanilla();
        assert_eq!(3, convert::janet_to_json(&client.run("(+ 1 2)").unwrap()));
    }

    #[test]
    fn test_gurp_client() {
        let client = gurp().unwrap();
        assert_eq!(3, convert::janet_to_json(&client.run("(+ 1 2)").unwrap()));

        assert_eq!(
            "/path/to/file",
            convert::janet_to_json(&client.run(r#"(pathcat "path" "to" "file")"#).unwrap())
        );

        assert_eq!(
            r#"{"a":123}"#,
            convert::janet_to_json(&client.run(r#"(to-json {:a 123})"#).unwrap())
        );
    }
}
