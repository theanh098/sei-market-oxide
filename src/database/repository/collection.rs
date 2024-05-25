use crate::{
    database::entity::{collection, prelude::Collection},
    server::api::collection::{SortBy, SortDirection},
    service::CollectionMetadata,
};
use sea_orm::{
    prelude::Decimal, query, sea_query::OnConflict, DatabaseBackend, DatabaseConnection, DbErr,
    EntityTrait, FromQueryResult, Set, Statement,
};
use serde_json::Value;

pub async fn find_by_address(
    db: &DatabaseConnection,
    address: &str,
) -> Result<Option<collection::Model>, DbErr> {
    Collection::find_by_id(address).one(db).await
}

pub async fn create(db: &DatabaseConnection, params: CreateCollectionParams) -> Result<(), DbErr> {
    let collection = collection::ActiveModel {
        address: Set(params.address),
        name: Set(params.name),
        symbol: Set(params.symbol),
        supply: Set(params.supply),
        description: Set(params.metadata.description),
        royalty: Set(params.royalty),
        banner: Set(params.metadata.banner),
        image: Set(params.metadata.pfp),
        socials: Set(params.metadata.socials),
        ..Default::default()
    };

    Collection::insert(collection)
        .on_conflict(
            OnConflict::column(collection::Column::Address)
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await?;

    Ok(())
}

pub async fn find_collections_with_stats(
    db: &DatabaseConnection,
    search: Option<String>,
    page: u64,
    limit: u8,
    _sort_by: SortBy,
    _sort_direction: SortDirection,
) -> Result<Vec<Value>, DbErr> {
    let skip = (page - 1) * limit as u64;

    let search = search
        .map(|s| format!("LOWER(name) LIKE '%{}%'", s.to_lowercase()))
        .unwrap_or("1 = 1".to_owned());

    let collections = query::JsonValue::find_by_statement(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        format!(
            "SELECT * FROM collection_view WHERE {} OFFSET $1 LIMIT $2;",
            search
        ),
        [skip.into(), limit.into()],
    ))
    .all(db)
    .await?;

    Ok(collections)
}

pub struct CreateCollectionParams {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub supply: i32,
    pub metadata: CollectionMetadata,
    pub royalty: Option<Decimal>,
}
