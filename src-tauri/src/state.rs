use std::process::Child;
use std::sync::{Arc};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

pub struct OAuthConfig {
    pub client_id: String,
    pub response_type: String,
    pub scope: String,
    pub tenant: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MinecraftAuthentication {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: String,
    pub profile: MinecraftProfile,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MinecraftProfile {
    pub id: String,
    pub name: String,
}

pub struct LaunchInstance {
    pub child: Arc<Mutex<Child>>,
}

impl LaunchInstance {
    pub async fn shutdown(&self) {
        self.child.lock().await.kill().expect("Failed to kill process");
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub enum RepositoryType {
    REMOTE,
    LOCAL,
}

impl RepositoryType {
    pub fn cli_arg(&self) -> &'static str {
        match self {
            RepositoryType::REMOTE => { "default" }
            RepositoryType::LOCAL => { "local" }
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Extension {
    pub descriptor: String,
    pub repository: String,
    pub repository_type: RepositoryType,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct Mod {
    pub project_id: String,
    pub loader: String,
}