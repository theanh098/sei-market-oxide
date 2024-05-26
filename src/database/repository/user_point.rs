use sea_orm::{
    prelude::DateTimeUtc,
    sea_query::{
        Alias, Expr, Func, NullOrdering, PostgresQueryBuilder, Query, WindowStatement,
    }, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbBackend, DbErr,
    EntityTrait, FromQueryResult, Order, Set, Statement,
};

use crate::database::{
    entity::{sea_orm_active_enums::LoyaltyPointKind, user_loyalty_point},
    model::{Count, LeaderboardParticipant},
};

pub async fn create(tx: &DatabaseTransaction, params: CreateUserPointParams) -> Result<(), DbErr> {
    let user_point = user_loyalty_point::ActiveModel {
        date: Set(params.date.into()),
        kind: Set(params.kind),
        point: Set(params.point),
        wallet_address: Set(params.wallet_address),
        ..Default::default()
    };

    user_loyalty_point::Entity::insert(user_point)
        .exec(tx)
        .await?;

    Ok(())
}

pub async fn find_leaderboad_by_date(
    db: &DatabaseConnection,
    from: DateTimeUtc,
    to: DateTimeUtc,
    page: u64,
    limit: u8,
    wallet_address: Option<String>,
) -> Result<
    (
        Vec<LeaderboardParticipant>,
        i64,
        Option<LeaderboardParticipant>,
    ),
    DbErr,
> {
    let mut user_on_leaderboard = None;

    let participants =
        find_leaderboard_participants_by_date(db, from, to, Some((page, limit)), None).await?;

    let total = count_leaderboard_participants_by_date(db, from, to).await?;

    if let Some(wallet_address) = wallet_address {
        user_on_leaderboard =
            find_leaderboard_participants_by_date(db, from, to, None, Some(wallet_address))
                .await?
                .get(0)
                .cloned();
    }

    Ok((participants, total, user_on_leaderboard))
}

async fn find_leaderboard_participants_by_date(
    db: &DatabaseConnection,
    from: DateTimeUtc,
    to: DateTimeUtc,
    paging: Option<(u64, u8)>,
    wallet_address: Option<String>,
) -> Result<Vec<LeaderboardParticipant>, DbErr> {
    let mut query = Query::select();

    query.expr(Expr::cust("*")).from_subquery(
        Query::select()
            .column(user_loyalty_point::Column::WalletAddress)
            .expr_as(user_loyalty_point::Column::Point.sum(), Alias::new("point"))
            .expr_window(
                Expr::cust("rank()"),
                WindowStatement::new()
                    .order_by_expr_with_nulls(
                        user_loyalty_point::Column::Point.sum(),
                        Order::Desc,
                        NullOrdering::Last,
                    )
                    .take(),
            )
            .from(user_loyalty_point::Entity)
            .and_where(user_loyalty_point::Column::Date.gte(from))
            .and_where(user_loyalty_point::Column::Date.lt(to))
            .and_where(user_loyalty_point::Column::Point.gt(0))
            .and_where(user_loyalty_point::Column::Kind.ne(LoyaltyPointKind::Xp))
            .group_by_col(user_loyalty_point::Column::WalletAddress)
            .order_by_expr_with_nulls(
                user_loyalty_point::Column::Point.sum(),
                Order::Desc,
                NullOrdering::Last,
            )
            .to_owned(),
        Alias::new("tmp"),
    );

    if let Some(wallet_address) = wallet_address {
        query.and_where(Expr::cust("tmp.wallet_address").eq(wallet_address));
    } else {
        if let Some((page, limit)) = paging {
            let offset = (page - 1) * limit as u64;
            query.limit(limit as u64);
            query.offset(offset);
        }
    }

    let participants = sea_orm::query::JsonValue::find_by_statement(Statement::from_string(
        DbBackend::Postgres,
        query.to_string(PostgresQueryBuilder),
    ))
    .into_model::<LeaderboardParticipant>()
    .all(db)
    .await?;

    Ok(participants)
}

async fn count_leaderboard_participants_by_date(
    db: &DatabaseConnection,
    from: DateTimeUtc,
    to: DateTimeUtc,
) -> Result<i64, DbErr> {
    let query = Query::select()
        .expr(Func::count("*"))
        .from_subquery(
            Query::select()
                .distinct()
                .column(user_loyalty_point::Column::WalletAddress)
                .and_where(user_loyalty_point::Column::Date.gte(from))
                .and_where(user_loyalty_point::Column::Date.lt(to))
                .and_where(user_loyalty_point::Column::Point.gt(0))
                .and_where(user_loyalty_point::Column::Kind.ne(LoyaltyPointKind::Xp))
                .from(user_loyalty_point::Entity)
                .to_owned(),
            Alias::new("tmp"),
        )
        .to_string(PostgresQueryBuilder);

    let total = sea_orm::query::JsonValue::find_by_statement(Statement::from_string(
        DbBackend::Postgres,
        query,
    ))
    .into_model::<Count>()
    .one(db)
    .await?
    .unwrap_or_default();

    Ok(total.count)
}

pub struct CreateUserPointParams {
    pub date: DateTimeUtc,
    pub kind: LoyaltyPointKind,
    pub wallet_address: String,
    pub point: i32,
}
