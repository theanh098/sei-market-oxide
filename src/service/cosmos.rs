use prost::{DecodeError, Message};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::json;
use tendermint::{hash::Algorithm, Hash};
use tendermint_rpc::{
    endpoint::{header_by_hash, tx, tx_search},
    query::Query,
    Client, HttpClient, Order,
};

use crate::r#static::PALLET_CONTRACT_ADDRESS;

pub struct CosmosClient(HttpClient);

#[derive(thiserror::Error, Debug)]
pub enum CosmosClientError {
    #[error("Rpc errors : {0}")]
    RpcError(String),

    #[error("Json Error")]
    JsonError(#[from] serde_json::Error),

    #[error("Tendermint RPC Error")]
    TendermintRpcError(#[from] tendermint_rpc::Error),

    #[error("Decode Error")]
    ProstDecodeError(#[from] DecodeError),

    #[error("Hash Error")]
    TendermintHashError(#[from] tendermint::error::Error),
}

impl CosmosClient {
    pub fn from(http_client: HttpClient) -> Self {
        Self(http_client)
    }

    pub async fn get_cw721_contract_info(
        &self,
        address: &str,
    ) -> Result<ContractInfo, CosmosClientError> {
        let msg = json!({
            "contract_info": {}
        });

        self.query_contract(address, msg).await
    }

    pub async fn get_cw721_contract_supply(
        &self,
        address: &str,
    ) -> Result<Supply, CosmosClientError> {
        let msg = json!({
            "num_tokens": {}
        });

        self.query_contract(address, msg).await
    }

    pub fn as_http(&self) -> &HttpClient {
        &self.0
    }

    pub async fn get_nft_info(
        &self,
        address: &str,
        token_id: &str,
    ) -> Result<NftInfo, CosmosClientError> {
        let msg = json!({
            "nft_info": {
                "token_id": token_id
            }
        });

        self.query_contract(address, msg).await
    }

    #[allow(dead_code)]
    pub async fn get_nft_owner(
        &self,
        address: &str,
        token_id: &str,
    ) -> Result<NftOwner, CosmosClientError> {
        let msg = json!({
            "owner_of": {
                "token_id": token_id
            }
        });

        self.query_contract(address, msg).await
    }

    pub async fn get_pallet_listing(
        &self,
        token_address: &str,
        token_id: &str,
    ) -> Result<PalletListing, CosmosClientError> {
        let msg = json!({
            "nft": {
                "address": token_address,
                "token_id": token_id
            }
        });

        self.query_contract(PALLET_CONTRACT_ADDRESS, msg).await
    }

    pub async fn get_tx(&self, tx_hash: &str) -> Result<tx::Response, CosmosClientError> {
        let tx_hash = Hash::from_bytes(Algorithm::Sha256, tx_hash.as_bytes())?;
        let tx = self.as_http().tx(tx_hash.to_owned(), false).await?;

        Ok(tx)
    }

    #[allow(dead_code)]
    pub async fn get_tx_header(
        &self,
        tx_hash: &str,
    ) -> Result<header_by_hash::Response, CosmosClientError> {
        let tx_hash = Hash::from_bytes(Algorithm::Sha256, tx_hash.as_bytes())?;
        let header = self.as_http().header_by_hash(tx_hash).await?;

        Ok(header)
    }

    #[allow(dead_code)]
    pub async fn search_tx(
        &self,
        query: Query,
        page: u32,
        per_page: u8,
    ) -> Result<tx_search::Response, CosmosClientError> {
        let res = self
            .as_http()
            .tx_search(query, false, page, per_page, Order::Ascending)
            .await?;

        Ok(res)
    }

    async fn query_contract<T, U>(&self, address: &str, msg: T) -> Result<U, CosmosClientError>
    where
        T: Serialize,
        U: DeserializeOwned,
    {
        let query = QueryContractRequest {
            address: address.to_string(),
            query_data: serde_json::to_vec(&msg)?,
        };

        let res = self
            .as_http()
            .abci_query(
                Some("/cosmwasm.wasm.v1.Query/SmartContractState".to_string()),
                query.encode_to_vec(),
                None,
                false,
            )
            .await?;

        if res.code.is_err() {
            return Err(CosmosClientError::RpcError(res.log));
        }

        let raw = QueryRawContractResponse::decode(res.value.as_slice())?;

        let res = serde_json::from_slice::<U>(raw.data.as_slice())?;

        Ok(res)
    }
}

#[derive(Deserialize, Debug)]
pub struct ContractInfo {
    pub name: String,
    pub symbol: String,
}

#[derive(Deserialize, Debug)]
pub struct Supply {
    pub count: u32,
}

#[derive(Deserialize, Debug)]
pub struct NftInfo {
    pub token_uri: String,
    pub extension: Option<Extension>,
}

#[derive(Deserialize, Debug)]
pub struct Extension {
    pub royalty_percentage: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct NftOwner {
    pub owner: String,
}

#[derive(Deserialize, Debug)]

pub struct PalletListing {
    pub owner: String,
    pub auction: Option<PalletAuction>,
}

#[derive(Deserialize, Debug)]

pub struct PalletAuction {
    pub created_at: u32,
    pub expiration_time: u32,
    pub prices: [Price; 1],
}

#[derive(Deserialize, Debug)]

pub struct Price {
    pub amount: String,
    pub denom: String,
}

#[derive(prost::Message)]
struct QueryContractRequest {
    #[prost(string, tag = "1")]
    address: prost::alloc::string::String,

    #[prost(bytes = "vec", tag = "2")]
    query_data: prost::alloc::vec::Vec<u8>,
}

#[derive(prost::Message)]
struct QueryRawContractResponse {
    #[prost(bytes = "vec", tag = "1")]
    pub data: prost::alloc::vec::Vec<u8>,
}
