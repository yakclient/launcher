use std::collections::HashMap;
use serde::Deserialize;
use crate::launch::minecraft::Error;
use crate::launch::minecraft::Error::Serde;

// All credit for the following code goes to the awesome people working
// on Modrinth :)

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LibraryPatch {
    #[serde(rename = "_comment")]
    pub _comment: String,
    #[serde(rename = "match")]
    pub match_: Vec<String>,
    pub additional_libraries: Option<Vec<crate::launch::minecraft::Library>>,
    #[serde(rename = "override")]
    pub override_: Option<PartialLibrary>,
    pub patch_additional_libraries: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PartialLibrary {
    pub downloads: Option<crate::launch::minecraft::LibraryDownloads>,
    pub extract: Option<crate::launch::minecraft::LibraryExtract>,
    pub name: Option<String>,
    pub natives: Option<HashMap<String, String>>,
    pub rules: Option<Vec<crate::launch::minecraft::Rule>>,
}

pub fn fetch_library_patches() -> Result<Vec<LibraryPatch>, Error> {
    let patches = include_bytes!("../../library-patches.json");
    Ok(serde_json::from_slice(patches).map_err(Serde)?)
}

pub fn patch_library(
    patches: &Vec<LibraryPatch>,
    mut library: crate::launch::minecraft::Library,
) -> Vec<crate::launch::minecraft::Library> {
    let mut val = Vec::new();

    let actual_patches = patches
        .iter()
        .filter(|x| x.match_.contains(&library.name))
        .collect::<Vec<_>>();

    if !actual_patches.is_empty() {
        for patch in actual_patches {
            if let Some(override_) = &patch.override_ {
                library = merge_partial_library(override_.clone(), library);
            }

            if let Some(additional_libraries) = &patch.additional_libraries {
                for additional_library in additional_libraries {
                    if patch.patch_additional_libraries.unwrap_or(false) {
                        let mut libs =
                            patch_library(patches, additional_library.clone());
                        val.append(&mut libs)
                    } else {
                        val.push(additional_library.clone());
                    }
                }
            }
        }

        val.push(library);
    } else {
        val.push(library);
    }

    val
}

pub fn merge_partial_library(
    partial: PartialLibrary,
    mut merge: crate::launch::minecraft::Library,
) -> crate::launch::minecraft::Library {
    if let Some(downloads) = partial.downloads {
        // if let Some(merge_downloads) = &mut merge.downloads {
        //     if let Some(artifact) = downloads.artifact {
        //         merge_downloads.artifact = Some(artifact);
        //     }
        //     if let Some(classifiers) = downloads.classifiers {
        //         if let Some(merge_classifiers) =
        //             &mut merge_downloads.classifiers
        //         {
        //             for classifier in classifiers {
        //                 merge_classifiers.insert(classifier.0, classifier.1);
        //             }
        //         } else {
        //             merge_downloads.classifiers = Some(classifiers);
        //         }
        //     }
        // } else {
            merge.downloads = downloads
        // }
    }
    if let Some(extract) = partial.extract {
        merge.extract = Some(extract)
    }
    if let Some(name) = partial.name {
        merge.name = name
    }
    if let Some(natives) = partial.natives {
        if let Some(merge_natives) = &mut merge.natives {
            for native in natives {
                merge_natives.insert(native.0, native.1);
            }
        } else {
            merge.natives = Some(natives);
        }
    }
    if let Some(rules) = partial.rules {
        if let Some(merge_rules) = &mut merge.rules {
            for rule in rules {
                merge_rules.push(rule);
            }
        } else {
            merge.rules = Some(rules)
        }
    }

    merge
}