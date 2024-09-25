use std::process::Child;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

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
    pub fn shutdown(&self) {
        self.child.lock().unwrap().kill().expect("Failed to kill process");
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Extension {
    pub descriptor: String,
    pub repository: String
}

pub type ExtensionState = Arc<Mutex<Vec<Extension>>>;