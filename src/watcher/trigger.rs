use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, Statement};

use crate::error::AppError;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Channel {
    StreamTx,
}

pub async fn create_stream_tx_trigger(db: &DatabaseConnection) -> Result<(), AppError> {
    let channel = Channel::STREAM_TX_CHANNEL;
    let fn_adapter = "stream_tx_fn()";

    let create_stream_tx_function_stm = format!(
        r#"
            CREATE OR REPLACE FUNCTION {}
            RETURNS TRIGGER AS $$
            BEGIN
                IF TG_OP = 'INSERT' OR TG_OP = 'UPDATE' THEN
                    PERFORM pg_notify('{}', row_to_json(NEW)::text);
                ELSE          
                    PERFORM pg_notify('{}', row_to_json(OLD)::text);
                END IF;

                RETURN NEW;
            END;
            $$ LANGUAGE plpgsql;
    "#,
        fn_adapter, channel, channel
    );

    let create_stream_tx_trigger_stm = format!(
        r#"
            CREATE OR REPLACE TRIGGER stream_tx_trigger 
            AFTER INSERT OR DELETE OR UPDATE OF is_failure, message
            ON stream_tx
            FOR EACH ROW EXECUTE PROCEDURE {};
    "#,
        fn_adapter
    );

    db.execute(Statement::from_string(
        DbBackend::Postgres,
        create_stream_tx_function_stm,
    ))
    .await?;

    db.execute(Statement::from_string(
        DbBackend::Postgres,
        create_stream_tx_trigger_stm,
    ))
    .await?;

    Ok(())
}

impl Channel {
    const STREAM_TX_CHANNEL: &'static str = "stream_tx_channel";

    pub fn to_str(self) -> &'static str {
        match self {
            Self::StreamTx => Self::STREAM_TX_CHANNEL,
        }
    }

    pub fn from_str(channel: &str) -> Self {
        match channel {
            Self::STREAM_TX_CHANNEL => Self::StreamTx,
            _ => panic!("unknown channel"),
        }
    }
}
