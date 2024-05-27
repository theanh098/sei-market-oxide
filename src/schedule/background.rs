use super::cronjob_expression::CronExpression;
use crate::error::AppError;
use chrono::Utc;
use cron::Schedule;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static::lazy_static! {
    static ref PROGRESS: Arc<RwLock<HashMap<&'static str, bool>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

type Worker<C> = dyn Fn(C) -> Pin<Box<dyn Future<Output = Result<(), AppError>>>>;

pub struct Background<'r, C> {
    context: Option<C>,
    workers: HashMap<&'static str, (CronExpression, &'r Worker<C>)>,
}

impl<'r, C> Background<'r, C>
where
    C: Clone,
{
    pub fn new() -> Self {
        Self {
            context: None,
            workers: HashMap::new(),
        }
    }

    pub fn set_context(mut self, context: C) -> Self {
        self.context = Some(context);
        self
    }

    pub fn add_job(
        mut self,
        name: &'static str,
        cron_expression: CronExpression,
        worker: &'r Worker<C>,
    ) -> Self {
        self.workers.insert(name, (cron_expression, worker));
        self
    }

    pub async fn start(&mut self) {
        let context = self.context.as_ref().unwrap();
        let mut futures = Vec::new();

        for (job, (cron_expression, worker)) in &self.workers {
            let con = context.clone();

            let schedule = Schedule::from_str(cron_expression.to_str()).unwrap();

            let mut progress = PROGRESS.write().await;

            let fut = async move {
                //  under the hood
                //  datetime that is the time the task will run in the future
                //  so from datetime we have the duration that system must be wait before executing the next task
                for datetime in schedule.upcoming(Utc) {
                    // datetime is expected in the future
                    // some reasons maybe make its become order than the time at now(),
                    // such as the task is expected to compelete before the next schedule, but it dose not
                    // so just skip those schedule
                    if let Ok(duration) = datetime.signed_duration_since(Utc::now()).to_std() {
                        tokio::time::sleep(duration).await;

                        let is_running = *progress.get(job).unwrap_or(&false);

                        if !is_running {
                            progress.insert(&job, true);

                            worker(con.clone()).await.unwrap_or_else(|e| {
                                eprintln!("error occur from cronjob::{} >> {}", job, e)
                            });
                            progress.insert(&job, false);
                        }
                    }
                }
            };

            futures.push(fut);
        }

        futures::future::join_all(futures).await;
    }
}
