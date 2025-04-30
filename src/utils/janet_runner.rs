use crate::doers::directory;
use janetrs::client::JanetClient;
use janetrs::env::CFunOptions;

pub fn janet_client() -> JanetClient {
    let mut client = JanetClient::init_with_default_env().expect("Failed to create Janet client");
    setup_bindings(&mut client);
    client
}

fn setup_bindings(client: &mut JanetClient) {
    client.add_c_fn(CFunOptions::new(c"directory-is", directory::directory_is_c));
    // client.add_c_fn(CFunOptions::new(c"setup-host", host::setup_host_c));
}
