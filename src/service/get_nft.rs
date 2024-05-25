use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
pub struct NftMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<String>,
    pub attributes: Option<Vec<NftAttribute>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NftAttribute {
    pub trait_type: Option<String>,
    pub r#type: Option<String>,
    pub value: Option<Value>,
    pub display_type: Option<Value>,
}

pub async fn get_nft_metadata(uri: &str) -> Result<NftMetadata, reqwest::Error> {
    reqwest::get(uri).await?.json::<NftMetadata>().await
}
