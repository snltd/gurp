use janetrs::client::JanetClient;

pub fn janet_client() -> JanetClient {
    tracing::debug!("Initialising janet client");
    JanetClient::init_with_default_env().expect("Failed to create Janet client")
}
