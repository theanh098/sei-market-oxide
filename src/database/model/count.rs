use sea_orm::FromQueryResult;

#[derive(FromQueryResult, Default)]
pub struct Count {
    pub count: i64,
}
