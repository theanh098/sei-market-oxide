mod background;
mod cronjob_expression;

use self::{background::Background, cronjob_expression::CronExpression};
use crate::error::AppError;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};

pub async fn background() {
    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("db_url must be set");

    let mut opt = ConnectOptions::new(db_url);
    opt.sqlx_logging(false);

    let db = Database::connect(opt).await.unwrap();

    Background::new()
        .set_context(db)
        // .add_job(CronExpression::EverySecond, &|db| {
        //     Box::pin(async move { run_per_second(db).await })
        // })
        // .add_job(CronExpression::Every5Seconds, &|db| {
        //     Box::pin(async move { run_per_5_seconds(db).await })
        // })
        .add_job("run??", CronExpression::Every10Seconds, &|db| {
            Box::pin(async move { run_per_10_seconds(db).await })
        })
        .start()
        .await;
}

// async fn run_per_second(_db: DatabaseConnection) -> Result<(), AppError> {
//     println!("run every 1 secord");
//     Ok(())
// }

// async fn run_per_5_seconds(_db: DatabaseConnection) -> Result<(), AppError> {
//     println!("run every 5 secords");
//     Ok(())
// }

async fn run_per_10_seconds(_db: DatabaseConnection) -> Result<(), AppError> {
    println!("run every 10 secords");
    Ok(())
}
