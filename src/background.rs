use chrono::Utc;
use cron::Schedule;
use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use tokio::time::sleep;

use crate::error::AppError;

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
                        sleep(duration).await;
                        let _ = handler(context.clone()).await;
                    }
                }
            });

            futures.push(f);
        }

        futures::future::join_all(futures).await;
    }
}

// type Worker<C> = dyn Fn(C) -> Pin<Box<dyn Future<Output = ()>>> + Sync + Send + 'static;

// pub struct Scheduler<C>
// where
//     C: Default + Clone + Sync + Send + 'static,
// {
//     context: Option<C>,
//     jobs: Vec<(CronExpression, &'static Worker<C>)>,
// }

// impl<'a, C: Default + Clone + Sync + Send + 'static> Scheduler<C> {
//     pub fn new() -> Self {
//         Self {
//             context: None,
//             jobs: Vec::new(),
//         }
//     }

//     pub fn set_context(mut self, context: C) -> Self {
//         self.context = Some(context);
//         self
//     }

//     pub fn add_job<J>(mut self, cron_expression: CronExpression, job: &'static J) -> Self
//     where
//         J: Fn(C) -> Pin<Box<dyn Future<Output = ()>>> + Sync + Send + 'static,
//     {
//         self.jobs.push((cron_expression, job));
//         self
//     }

//     pub async fn start(self) -> Result<(), AppError> {
//         let context: C = self.context.unwrap_or_default();

//         // let mut futures = Vec::new();

//         for (cron, job) in self.jobs {
//             let con = context.clone();

// let schedule = Schedule::from_str(cron.to_str())
//     .map_err(|e| AppError::Unexpected(e.to_string()))?;

//             tokio::spawn(async move {
//                 for datetime in schedule.upcoming(Utc) {
//                     if let Ok(duration) = datetime.signed_duration_since(Utc::now()).to_std() {
//                         sleep(duration).await;
//                         job(con.clone()).await;
//                     }
//                 }
//             });

//             // let fut = async move {
//             //     for datetime in schedule.upcoming(Utc) {
//             //         let now = Utc::now();

//         if let Ok(duration) = datetime.signed_duration_since(now).to_std() {
//             sleep(duration).await;
//             job(con.clone()).await;
//         }
//             //     }
//             // };

//             // futures.push(fut);
//         }

//         // futures::future::join_all(futures).await;

//         Ok(())
//     }
// }

pub enum CronExpression {
    EverySecond,
    Every5Seconds,
    Every10Seconds,
    Every30Seconds,
    EveryMinute,
    Every5Minutes,
    Every10Minutes,
    Every30Minutes,
    EveryHour,
    Every2Hours,
    Every3Hours,
    Every4Hours,
    Every5Hours,
    Every6Hours,
    Every7Hours,
    Every8Hours,
    Every9Hours,
    Every10Hours,
    Every11Hours,
    Every12Hours,
    EveryDayAt1AM,
    EveryDayAt2AM,
    EveryDayAt3AM,
    EveryDayAt4AM,
    EveryDayAt5AM,
    EveryDayAt6AM,
    EveryDayAt7AM,
    EveryDayAt8AM,
    EveryDayAt9AM,
    EveryDayAt10AM,
    EveryDayAt11AM,
    EveryDayAtNoon,
    EveryDayAt1PM,
    EveryDayAt2PM,
    EveryDayAt3PM,
    EveryDayAt4PM,
    EveryDayAt5PM,
    EveryDayAt6PM,
    EveryDayAt7PM,
    EveryDayAt8PM,
    EveryDayAt9PM,
    EveryDayAt10PM,
    EveryDayAt11PM,
    EveryDayAtMidnight,
    EveryWeek,
    EveryWeekday,
    EveryWeekend,
    Every1stDayOfMonthAtMidnight,
    Every1stDayOfMonthAtNoon,
    Every2ndHour,
    Every2ndHourFrom1AMThrough11PM,
    Every2ndMonth,
    EveryQuarter,
    Every6Months,
    EveryYear,
    Every30MinutesBetween9AMAnd5PM,
    Every30MinutesBetween9AMAnd6PM,
    Every30MinutesBetween10AMAnd7PM,
    MondayToFridayAt1AM,
    MondayToFridayAt2AM,
    MondayToFridayAt3AM,
    MondayToFridayAt4AM,
    MondayToFridayAt5AM,
    MondayToFridayAt6AM,
    MondayToFridayAt7AM,
    MondayToFridayAt8AM,
    MondayToFridayAt9AM,
    MondayToFridayAt9_30AM,
    MondayToFridayAt10AM,
    MondayToFridayAt11AM,
    MondayToFridayAt11_30AM,
    MondayToFridayAt12PM,
    MondayToFridayAt1PM,
    MondayToFridayAt2PM,
    MondayToFridayAt3PM,
    MondayToFridayAt4PM,
    MondayToFridayAt5PM,
    MondayToFridayAt6PM,
    MondayToFridayAt7PM,
    MondayToFridayAt8PM,
    MondayToFridayAt9PM,
    MondayToFridayAt10PM,
    MondayToFridayAt11PM,
}

impl CronExpression {
    fn to_str(&self) -> &'static str {
        match self {
            CronExpression::EverySecond => "* * * * * *",
            CronExpression::Every5Seconds => "*/5 * * * * *",
            CronExpression::Every10Seconds => "*/10 * * * * *",
            CronExpression::Every30Seconds => "*/30 * * * * *",
            CronExpression::EveryMinute => "*/1 * * * *",
            CronExpression::Every5Minutes => "0 */5 * * * *",
            CronExpression::Every10Minutes => "0 */10 * * * *",
            CronExpression::Every30Minutes => "0 */30 * * * *",
            CronExpression::EveryHour => "0 0-23/1 * * *",
            CronExpression::Every2Hours => "0 0-23/2 * * *",
            CronExpression::Every3Hours => "0 0-23/3 * * *",
            CronExpression::Every4Hours => "0 0-23/4 * * *",
            CronExpression::Every5Hours => "0 0-23/5 * * *",
            CronExpression::Every6Hours => "0 0-23/6 * * *",
            CronExpression::Every7Hours => "0 0-23/7 * * *",
            CronExpression::Every8Hours => "0 0-23/8 * * *",
            CronExpression::Every9Hours => "0 0-23/9 * * *",
            CronExpression::Every10Hours => "0 0-23/10 * * *",
            CronExpression::Every11Hours => "0 0-23/11 * * *",
            CronExpression::Every12Hours => "0 0-23/12 * * *",
            CronExpression::EveryDayAt1AM => "0 01 * * *",
            CronExpression::EveryDayAt2AM => "0 02 * * *",
            CronExpression::EveryDayAt3AM => "0 03 * * *",
            CronExpression::EveryDayAt4AM => "0 04 * * *",
            CronExpression::EveryDayAt5AM => "0 05 * * *",
            CronExpression::EveryDayAt6AM => "0 06 * * *",
            CronExpression::EveryDayAt7AM => "0 07 * * *",
            CronExpression::EveryDayAt8AM => "0 08 * * *",
            CronExpression::EveryDayAt9AM => "0 09 * * *",
            CronExpression::EveryDayAt10AM => "0 10 * * *",
            CronExpression::EveryDayAt11AM => "0 11 * * *",
            CronExpression::EveryDayAtNoon => "0 12 * * *",
            CronExpression::EveryDayAt1PM => "0 13 * * *",
            CronExpression::EveryDayAt2PM => "0 14 * * *",
            CronExpression::EveryDayAt3PM => "0 15 * * *",
            CronExpression::EveryDayAt4PM => "0 16 * * *",
            CronExpression::EveryDayAt5PM => "0 17 * * *",
            CronExpression::EveryDayAt6PM => "0 18 * * *",
            CronExpression::EveryDayAt7PM => "0 19 * * *",
            CronExpression::EveryDayAt8PM => "0 20 * * *",
            CronExpression::EveryDayAt9PM => "0 21 * * *",
            CronExpression::EveryDayAt10PM => "0 22 * * *",
            CronExpression::EveryDayAt11PM => "0 23 * * *",
            CronExpression::EveryDayAtMidnight => "0 0 * * *",
            CronExpression::EveryWeek => "0 0 * * 0",
            CronExpression::EveryWeekday => "0 0 * * 1-5",
            CronExpression::EveryWeekend => "0 0 * * 6,0",
            CronExpression::Every1stDayOfMonthAtMidnight => "0 0 1 * *",
            CronExpression::Every1stDayOfMonthAtNoon => "0 12 1 * *",
            CronExpression::Every2ndHour => "0 */2 * * *",
            CronExpression::Every2ndHourFrom1AMThrough11PM => "0 1-23/2 * * *",
            CronExpression::Every2ndMonth => "0 0 1 */2 *",
            CronExpression::EveryQuarter => "0 0 1 */3 *",
            CronExpression::Every6Months => "0 0 1 */6 *",
            CronExpression::EveryYear => "0 0 1 0 *",
            CronExpression::Every30MinutesBetween9AMAnd5PM => "0 */30 9-17 * * *",
            CronExpression::Every30MinutesBetween9AMAnd6PM => "0 */30 9-18 * * *",
            CronExpression::Every30MinutesBetween10AMAnd7PM => "0 */30 10-19 * * *",
            CronExpression::MondayToFridayAt1AM => "0 0 01 * * 1-5",
            CronExpression::MondayToFridayAt2AM => "0 0 02 * * 1-5",
            CronExpression::MondayToFridayAt3AM => "0 0 03 * * 1-5",
            CronExpression::MondayToFridayAt4AM => "0 0 04 * * 1-5",
            CronExpression::MondayToFridayAt5AM => "0 0 05 * * 1-5",
            CronExpression::MondayToFridayAt6AM => "0 0 06 * * 1-5",
            CronExpression::MondayToFridayAt7AM => "0 0 07 * * 1-5",
            CronExpression::MondayToFridayAt8AM => "0 0 08 * * 1-5",
            CronExpression::MondayToFridayAt9AM => "0 0 09 * * 1-5",
            CronExpression::MondayToFridayAt9_30AM => "0 30 09 * * 1-5",
            CronExpression::MondayToFridayAt10AM => "0 0 10 * * 1-5",
            CronExpression::MondayToFridayAt11AM => "0 0 11 * * 1-5",
            CronExpression::MondayToFridayAt11_30AM => "0 30 11 * * 1-5",
            CronExpression::MondayToFridayAt12PM => "0 0 12 * * 1-5",
            CronExpression::MondayToFridayAt1PM => "0 0 13 * * 1-5",
            CronExpression::MondayToFridayAt2PM => "0 0 14 * * 1-5",
            CronExpression::MondayToFridayAt3PM => "0 0 15 * * 1-5",
            CronExpression::MondayToFridayAt4PM => "0 0 16 * * 1-5",
            CronExpression::MondayToFridayAt5PM => "0 0 17 * * 1-5",
            CronExpression::MondayToFridayAt6PM => "0 0 18 * * 1-5",
            CronExpression::MondayToFridayAt7PM => "0 0 19 * * 1-5",
            CronExpression::MondayToFridayAt8PM => "0 0 20 * * 1-5",
            CronExpression::MondayToFridayAt9PM => "0 0 21 * * 1-5",
            CronExpression::MondayToFridayAt10PM => "0 0 22 * * 1-5",
            CronExpression::MondayToFridayAt11PM => "0 0 23 * * 1-5",
        }
    }
}
