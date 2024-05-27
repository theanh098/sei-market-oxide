mod background;
mod cronjob_expression;

use self::{background::Background, cronjob_expression::CronExpression};
use sea_orm::{ConnectOptions, Database};

pub async fn background() {
    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("db_url must be set");

    let mut opt = ConnectOptions::new(db_url);
    opt.sqlx_logging(false);

    let db = Database::connect(opt).await.unwrap();

    Background::new()
        .set_context(db)
        .add_job(CronExpression::Every10Seconds, |_db| async {
            println!("hello kitty");

            Ok(())
        })
        .start()
        .await;

    // loop {}
}
