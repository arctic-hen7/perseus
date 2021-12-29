use crate::errors::*;
use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

/// An immutable storage system. This wraps filesystem calls in a sensible asynchronous API, allowing abstraction of the base path
/// to a distribution directory or the like. Perseus uses this to store assts created at build time that won't change, which is
/// anything not involved in the *revalidation* or *incremental generation* strategies.
///
/// Note: the `.write()` methods on this implementation will create any missing parent directories automatically.
#[derive(Clone)]
pub struct ImmutableStore {
    root_path: String,
}
impl ImmutableStore {
    /// Creates a new immutable store. You should provide a path like `dist/` here.
    pub fn new(root_path: String) -> Self {
        Self { root_path }
    }
    /// Reads the given asset from the filesystem asynchronously.
    pub async fn read(&self, name: &str) -> Result<String, StoreError> {
        let asset_path = format!("{}/{}", self.root_path, name);
        let mut file = File::open(&asset_path)
            .await
            .map_err(|err| StoreError::ReadFailed {
                name: asset_path.clone(),
                source: err.into(),
            })?;
        let metadata = file.metadata().await;

        match metadata {
            Ok(_) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents)
                    .await
                    .map_err(|err| StoreError::ReadFailed {
                        name: asset_path,
                        source: err.into(),
                    })?;
                Ok(contents)
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                Err(StoreError::NotFound { name: asset_path })
            }
            Err(err) => Err(StoreError::ReadFailed {
                name: asset_path,
                source: err.into(),
            }),
        }
    }
    /// Writes the given asset to the filesystem asynchronously. This must only be used at build-time, and must not be changed
    /// afterward.
    pub async fn write(&self, name: &str, content: &str) -> Result<(), StoreError> {
        let asset_path = format!("{}/{}", self.root_path, name);
        let mut dir_tree: Vec<&str> = asset_path.split('/').collect();
        dir_tree.pop();

        create_dir_all(dir_tree.join("/"))
            .await
            .map_err(|err| StoreError::WriteFailed {
                name: asset_path.clone(),
                source: err.into(),
            })?;

        // This will either create the file or truncate it if it already exists
        let mut file = File::create(&asset_path)
            .await
            .map_err(|err| StoreError::WriteFailed {
                name: asset_path.clone(),
                source: err.into(),
            })?;
        file.write_all(content.as_bytes())
            .await
            .map_err(|err| StoreError::WriteFailed {
                name: asset_path.clone(),
                source: err.into(),
            })?;
        // TODO Can we use `sync_data()` here to reduce I/O?
        file.sync_all()
            .await
            .map_err(|err| StoreError::WriteFailed {
                name: asset_path,
                source: err.into(),
            })?;

        Ok(())
    }
}
