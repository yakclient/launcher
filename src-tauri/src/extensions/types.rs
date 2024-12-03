use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// Represents the YakClient ERM (or Extension Runtime Model)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionRuntimeModel {
    pub api_version: i32,
    pub group_id: String,
    pub name: String,
    pub version: String,

    pub repositories: Vec<HashMap<String, String>>,
    pub parents: Vec<ExtensionParent>,
    pub partitions: Vec<PartitionModelReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionModelReference {
    pub r#type: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionParent {
    pub group: String,
    pub extension: String,
    pub version: String,
}

// impl ExtensionParent {
//     pub fn to_descriptor(&self) -> ExtensionDescriptor {
//         ExtensionDescriptor {
//             group: self.group.clone(),
//             extension: self.extension.clone(),
//             version: self.version.clone(),
//         }
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(rename_all = "PascalCase")]
// pub struct ExtensionDescriptor {
//     pub group: String,
//     pub extension: String,
//     pub version: String,
// }
//
// impl ExtensionDescriptor {
//     pub fn parse_descriptor(descriptor: &str) -> Self {
//         let parts: Vec<&str> = descriptor.split(':').collect();
//         if parts.len() != 3 {
//             panic!("Invalid descriptor format: {}", descriptor);
//         }
//         Self {
//             group: parts[0].to_string(),
//             extension: parts[1].to_string(),
//             version: parts[2].to_string(),
//         }
//     }
// }

// impl ExtensionRuntimeModel {
//     pub fn descriptor(&self) -> ExtensionDescriptor {
//         ExtensionDescriptor::parse_descriptor(&format!(
//             "{}:{}:{}",
//             self.group_id, self.name, self.version
//         ))
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionRepository {
    pub r#type: String,
    pub settings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionRuntimeModel {
    pub r#type: String,
    pub name: String,
    pub repositories: Vec<ExtensionRepository>,
    pub dependencies: Vec<HashMap<String, String>>,
    pub options: HashMap<String, String>,
}
