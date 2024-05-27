use chrono::Utc;
use cron::Schedule;

use super::cronjob_expression::CronExpression;
use crate::error::AppError;
use std::collections::HashMap;
use std::future::Future;
use std::str::FromStr;

pub struct Background<C, F> {
    context: Option<C>,
    jobs: HashMap<&'static str, F>,
}

impl<C, F, Fut> Background<C, F>
where
    C: Clone + Send + 'static,
    F: Fn(C) -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), AppError>> + Send,
{
    pub fn new() -> Self {
        Self {
            context: None,
            jobs: HashMap::new(),
        }
    }

    pub fn set_context(mut self, context: C) -> Self {
        self.context = Some(context);
        self
    }

    pub fn add_job(mut self, expression: CronExpression, hanlder: F) -> Self {
        self.jobs.insert(expression.to_str(), hanlder);
        self
    }

    pub async fn start(self) {
        let Self { context, jobs } = self;
        let context = context.unwrap();

        let mut futures = Vec::with_capacity(12);

        for (timeline, handler) in jobs.into_iter() {
            let context = context.clone();

            let schedule = Schedule::from_str(timeline)
                .map_err(|e| AppError::Unexpected(e.to_string()))
                .unwrap();

            let f = tokio::spawn(async move {
                for datetime in schedule.upcoming(Utc) {
                    if let Ok(duration) = datetime.signed_duration_since(Utc::now()).to_std() {
                        tokio::time::sleep(duration).await;
                        let _ = handler(context.clone()).await;
                    }
                }
            });

            futures.push(f);
        }

        futures::future::join_all(futures).await;
    }
}
