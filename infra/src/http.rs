use std::time::Duration;

use derive_more::{Deref, Into};
use reqwest::Client;

#[derive(Clone, Deref, Into)]
pub struct AuthClient(reqwest::Client);

impl AuthClient {
    pub fn new() -> reqwest::Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(5))
            .https_only(cfg!(not(debug_assertions)))
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        Ok(Self(client))
    }
}

#[derive(Clone, Deref, Into)]
pub struct HttpClient(reqwest::Client);

impl HttpClient {
    pub fn new() -> reqwest::Result<Self> {
        let client = Client::builder().build()?;
        Ok(Self(client))
    }
}
