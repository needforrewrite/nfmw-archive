use s3::{Bucket, Region, creds::Credentials};
use uuid::Uuid;

use crate::database::assets::AssetType;

pub struct AssetStore {
    bucket: Box<Bucket>,
}

impl AssetStore {
    pub fn new() -> anyhow::Result<Self> {
        let region = Region::Custom {
            region: std::env::var("B2_REGION")?,        // e.g. "us-west-004"
            endpoint: std::env::var("B2_ENDPOINT")?,     // e.g. "https://s3.us-west-004.backblazeb2.com"
        };

        // Reads B2_KEY_ID / application key from env vars
        // (AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY, or pass explicitly)
        let credentials = Credentials::new(
            Some(&std::env::var("B2_KEY_ID")?),
            Some(&std::env::var("B2_APP_KEY")?),
            None, None, None,
        )?;

        let bucket_name = std::env::var("B2_BUCKET")?;
        let mut bucket = Bucket::new(&bucket_name, region, credentials)?;
        bucket.set_path_style(); // recommended for B2

        Ok(Self { bucket })
    }

    /// Build the object key for a new asset. Persist the returned key
    /// (and the UUID) in your database alongside the asset's metadata.
    pub fn new_key(kind: AssetType) -> (Uuid, String) {
        let id = Uuid::new_v4();
        let key = format!("assets/{}/{}", kind.to_string(), id);
        (id, key)
    }

    pub async fn put(&self, key: &str, data: &[u8], content_type: &str) -> anyhow::Result<()> {
        let resp = self
            .bucket
            .put_object_with_content_type(key, data, content_type)
            .await?;
        if resp.status_code() != 200 {
            anyhow::bail!("B2 put failed: status {}", resp.status_code());
        }
        Ok(())
    }

    pub async fn get(&self, key: &str) -> anyhow::Result<Vec<u8>> {
        let resp = self.bucket.get_object(key).await?;
        if resp.status_code() != 200 {
            anyhow::bail!("B2 get failed: status {}", resp.status_code());
        }
        Ok(resp.bytes().to_vec())
    }

    pub async fn delete(&self, key: &str) -> anyhow::Result<()> {
        let resp = self.bucket.delete_object(key).await?;
        if !(200..=299).contains(&resp.status_code()) {
            anyhow::bail!("B2 delete failed: status {}", resp.status_code());
        }
        Ok(())
    }
}