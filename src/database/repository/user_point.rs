use sea_orm::{prelude::DateTimeUtc, DatabaseTransaction, DbErr, EntityTrait, Set};

use crate::database::entity::{
    prelude::UserLoyaltyPoint, sea_orm_active_enums::LoyaltyPointKind, user_loyalty_point,
};

pub async fn create(tx: &DatabaseTransaction, params: CreateUserPointParams) -> Result<(), DbErr> {
    let user_point = user_loyalty_point::ActiveModel {
        date: Set(params.date.into()),
        kind: Set(params.kind),
        point: Set(params.point),
        wallet_address: Set(params.wallet_address),
        ..Default::default()
    };

    UserLoyaltyPoint::insert(user_point).exec(tx).await?;

    Ok(())
}

pub struct CreateUserPointParams {
    pub date: DateTimeUtc,
    pub kind: LoyaltyPointKind,
    pub wallet_address: String,
    pub point: i32,
}
