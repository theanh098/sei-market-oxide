use sea_orm::{prelude::Decimal, FromQueryResult};

#[derive(FromQueryResult, Default, Clone)]
pub struct LeaderboardParticipant {
    pub wallet_address: String,
    pub rank: i64,
    pub point: Decimal,
}
