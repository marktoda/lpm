use anyhow::Result;
use crev_recursive_digest;
use log::debug;
use log::error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub struct _Cache {
    last_updated_version: HashMap<PathBuf, String>,
}

impl _Cache {
    pub fn _new() -> _Cache {
        _Cache {
            last_updated_version: HashMap::new(),
        }
    }

    pub fn _has_changed(&self, path: PathBuf) -> bool {
        let hash = match _get_package_hash(path.clone()) {
            Ok(digest) => digest,
            Err(e) => {
                error!("Error getting package hash: {:?}", e);
                return false;
            }
        };
        self.last_updated_version
            .get(&path)
            .map_or(false, |version| version.clone() == hash)
    }

    pub fn _update(&mut self, path: PathBuf) -> Result<()> {
        let digest = _get_package_hash(path.clone())?;
        self.last_updated_version.insert(path.clone(), digest);
        Ok(())
    }
}

pub fn _get_package_hash(path: PathBuf) -> Result<String> {
    let rdigest = crev_recursive_digest::RecursiveDigest::<blake2::Blake2b, _, _>::new()
        .additional_data(|entry, writer| {
            let metadata = entry.metadata()?;
            writer.input(&_metadata_to_u16(&metadata).to_be_bytes());
            Ok(())
        })
        .build();
    let digest = rdigest.get_digest_of(&path)?;
    debug!("Digest: {:?} for {}", hex::encode(&digest), path.display());

    Ok(hex::encode(&digest))
}

#[cfg(unix)]
fn _metadata_to_u16(metadata: &std::fs::Metadata) -> u16 {
    let permissions = metadata.permissions();
    use std::os::unix::fs::PermissionsExt;
    (permissions.mode() & 0x1ff) as u16
}

#[cfg(not(unix))]
fn _metadata_to_u16(metadata: &std::fs::Metadata) -> u16 {
    let permissions = metadata.permissions();
    // TODO: what else to do on Windows?
    match (permissions.readonly(), metadata.is_dir()) {
        (false, false) => 0o444u16,
        (false, true) => 0o555,
        (true, false) => 0x666,
        (true, true) => 0x777,
    }
}
