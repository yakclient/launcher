use std::ops::Deref;
use std::process::Child;
use std::sync::{Arc, Mutex};
use serde::Deserialize;

pub struct OAuthConfig<'a> {
    pub client_id: &'a str,
    pub response_type: &'a str,
    pub scope: &'a str,
    pub tenant: &'a str,
}

pub struct MinecraftAuthentication {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub profile: MinecraftProfile
}

#[derive(Deserialize)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
}

pub struct LaunchInstance {
    pub child: Arc<Mutex<Child>>
}

impl LaunchInstance {
    // fn new(child: Child) -> LaunchInstance {
    //
    // }

    pub fn shutdown(&self) {
        self.child.lock().unwrap().kill().expect("Failed to kill process");
    }
}