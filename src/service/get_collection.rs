use crate::PALLET_API_URL;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CollectionMetadata {
    pub pfp: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub banner: Option<String>,
    pub socials: Option<serde_json::Value>,
}

pub async fn get_collection_metadata(address: &str) -> Result<CollectionMetadata, reqwest::Error> {
    let endpoint = format!("{}/v2/nfts/{address}/details", PALLET_API_URL);

    reqwest::get(endpoint)
        .await?
        .json::<CollectionMetadata>()
        .await
}
