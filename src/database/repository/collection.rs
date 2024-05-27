use crate::{
    database::{
        entity::{collection, collection_view},
        model::Count,
    },
    server::{api::collection::SortBy, deserialization::SortDirection},
    service::CollectionMetadata,
};
use sea_orm::{
    prelude::Decimal, sea_query::OnConflict, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QueryOrder, QuerySelect, QueryTrait, Set,
};

pub async fn find_by_address(
    db: &DatabaseConnection,
    address: &str,
) -> Result<Option<collection::Model>, DbErr> {
    collection::Entity::find_by_id(address).one(db).await
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

    collection::Entity::insert(collection)
        .on_conflict(
            OnConflict::column(collection::Column::Address)
                .do_nothing()
                .to_owned(),
        )
        .exec(db)
        .await
        .map(|_| ())
        .or_else(|error| {
            if let DbErr::RecordNotInserted = error {
                Ok(())
            } else {
                Err(error)
            }
        })
}

pub async fn find_collections_with_stats(
    db: &DatabaseConnection,
    search: Option<String>,
    page: u64,
    limit: u8,
    sort_by: SortBy,
    sort_direction: SortDirection,
) -> Result<(Vec<collection_view::Model>, i64), DbErr> {
    let skip = (page - 1) * limit as u64;

    let sort_by = match sort_by {
        SortBy::All => collection_view::Column::Volume,
        SortBy::_1h => collection_view::Column::VolumeOf1h,
        SortBy::_24h => collection_view::Column::VolumeOf24h,
        SortBy::_7d => collection_view::Column::VolumeOf7d,
        SortBy::_30d => collection_view::Column::VolumeOf30d,
    };

    let collections = collection_view::Entity::find()
        .apply_if(search.as_ref(), |query, search| {
            query.filter(collection_view::Column::Name.contains(search))
        })
        .order_by(sort_by, sort_direction.into_order())
        .limit(limit as u64)
        .offset(skip)
        .all(db)
        .await?;

    let total = collection::Entity::find()
        .select_only()
        .column_as(collection::Column::Address.count(), "count")
        .apply_if(search, |query, search| {
            query.filter(collection::Column::Name.contains(search))
        })
        .into_model::<Count>()
        .one(db)
        .await?
        .unwrap_or_default();

    Ok((collections, total.count))
}

pub struct CreateCollectionParams {
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub supply: i32,
    pub metadata: CollectionMetadata,
    pub royalty: Option<Decimal>,
}
