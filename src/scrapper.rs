#![allow(dead_code)]
use std::{collections::HashMap, time::Duration};

use crate::{
    database::{self},
    error::Error,
    repository::Repository,
    types::{Activity, Props, RawStats, StravaActivity},
    utils::{self, elapsed_time_to_sec, pace_to_sec},
};
use chrono::{Datelike, Utc};
use convert_case::{Case, Casing};
use futures::stream::{FuturesUnordered, StreamExt};
use regex::Regex;
use reqwest::Client;
use select::{
    document::Document,
    predicate::{Class, Name},
};
use tokio::task;

const BATCH_SIZE: usize = 300;
const MAX_CONCURRENT_TASKS: usize = 50;

lazy_static! {
    static ref REPO: Repository = Repository::new(database::clone());
}

#[derive(Default, Clone)]
pub struct Scrapper;

impl Scrapper {
    pub async fn run_worker(
        &self,
        arg_offset: i64,
        arg_limit: Option<i64>,
        jobs_count: Option<i64>,
        job_id: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut repo = REPO.clone();
        let mut futures = FuturesUnordered::new();

        if arg_offset < 0 {
            error!("Offset is negative");
            return Ok(());
        }

        let (_, total_records) = repo.get_users(1, 0).await?;

        let (arg_offset, mut max_offset) = if let Some(count) = jobs_count {
            let limit = total_records / count;
            let mut offset = job_id * limit;
            let mut max_offset = offset + limit;

            if offset > 0 {
                offset += 1
            }

            if job_id == count - 1 {
                max_offset = total_records;
            }

            (offset, max_offset)
        } else {
            (arg_offset, total_records)
        };

        if let Some(limit) = arg_limit {
            max_offset = std::cmp::min(max_offset, arg_offset + limit);
        }

        let batch_size = if max_offset < BATCH_SIZE as i64 {
            max_offset / 4
        } else {
            BATCH_SIZE as i64
        };

        info!(
            "[JOB-{job_id}] Starting scrapper {arg_offset}/{max_offset} of {total_records} records"
        );

        for offset in (arg_offset..max_offset).step_by(batch_size as usize) {
            let mut user_repo = repo.clone();
            let task = task::spawn(async move {
                let (users, _) = user_repo
                    .get_users(batch_size, offset)
                    .await
                    .expect("Failed to get users");

                let mut strava_ids = HashMap::new();

                for (_, user) in users.iter().enumerate() {
                    let id = user.strava_id.clone();
                    if id.is_empty() || id == "-" {
                        continue;
                    }

                    if let Ok(activities) = Self::run_scrapper(&id, job_id).await {
                        info!(
                            "[JOB-{job_id}][{id}] Found {} run activities",
                            activities.clone().len()
                        );

                        let result =
                            futures::future::join_all(activities.iter().map(async |activity| {
                                let output = Self::parse_activity(activity, job_id).await;
                                tokio::time::sleep(Duration::from_secs(2)).await;

                                output
                            }))
                            .await
                            .into_iter()
                            .filter_map(Result::ok)
                            .collect::<Vec<StravaActivity>>();

                        match user_repo.create_activities(result).await {
                            Ok(inserted) => {
                                info!(
                                    "[JOB-{job_id}][{id}] Inserted {inserted} activities for athlete {id}"
                                );
                                strava_ids.insert(id, inserted);
                            }
                            Err(e) => {
                                error!("[JOB-{job_id}][{id}] Failed to insert activities: {}", e);
                            }
                        }
                    } else {
                        warn!("[JOB-{job_id}][{id}] No activity data found");
                    }

                    tokio::time::sleep(Duration::from_secs(1)).await;
                }

                strava_ids
            });

            futures.push(task);

            // Throttle number of concurrent tasks
            if futures.len() >= MAX_CONCURRENT_TASKS {
                futures.next().await; // Wait for one to finish
            }
        }

        while let Some(x) = futures.next().await {
            match x {
                Ok(ids) => {
                    let keys = ids.keys().count();
                    let values = ids.values().sum::<usize>();

                    info!(
                        "[JOB-{job_id}][i] Finished processing athletes {keys} with {values} activities",
                    );
                }
                Err(e) => {
                    error!("[JOB-{job_id}] TaskError: {}", e);
                }
            }
        }

        info!("[JOB-{job_id}][i] Finished scrapper");

        Ok(())
    }

    async fn run_scrapper(athlete_id: &str, job_id: i64) -> Result<Vec<Activity>, Error> {
        let now = Utc::now().naive_utc();
        let year = now.year();
        let month = now.month();
        let url = format!(
            "https://www.strava.com/athletes/{athlete_id}?chart_type=miles&interval_type=month&interval={year}{month:02}&year_offset=0"
        );

        let mut client: Client;
        let mut res: reqwest::Response;
        let mut attempt = 1;

        loop {
            client = utils::build_client(None).await;
            res = client.get(url.clone()).send().await?;

            let status = res.headers().get("status").ok_or_else(|| 0).is_ok();
            if !status {
                error!("[JOB-{job_id}][{athlete_id}][SKIP] Failed to fetch athlete ID");
                return Err(error_custom!("Failed to fetch athlete"));
            }

            attempt += 1;
            if attempt > 5 {
                error!("[JOB-{job_id}][{athlete_id}][SKIP] Failed to fetch athlete ID");
                return Err(error_custom!("Failed to fetch athlete"));
            }

            if res.status().is_success() {
                break;
            }
        }

        let cookie = res.headers().get("cookie").cloned();
        let status_str = res.status().to_string();

        let html = res.text().await?;
        let document = Document::from_read(html.as_bytes())?;
        let body = document.find(Name("body")).next().unwrap();

        if !body.attr("class").unwrap_or_default().contains("logged-in") {
            error!("[JOB-{job_id}][SKIP] Not logged in: Session expired! ID: {athlete_id}");
            error!("[JOB-{job_id}][!!!!!!!!] Cookie: {status_str} {cookie:?}");
            return Err(error_custom!("Not logged in"));
        }

        let el_activity = body.find(Class("react-feed-component")).next();
        if el_activity.is_none() {
            return Err(error_custom!("No activity data found"));
        }

        if let Some(json_str) = el_activity.unwrap().attr("data-react-props") {
            let data = serde_json::from_str::<Props>(&json_str);

            if data.is_err() {
                error!(
                    "[JOB-{job_id}][{athlete_id}][SKIP] Failed to parse activity data from athlete "
                );
                error!("[JOB-{job_id}] Payload: {json_str}");
                return Err(error_custom!("Failed to parse activity data"));
            }

            let activities = data
                .unwrap()
                .app_context
                .entries
                .clone()
                .into_iter()
                .filter(|a| {
                    vec!["Activity", "GroupActivity"].contains(&a.entity.as_str())
                        && a.activity.clone().is_some_and(|a| a.activity_type == "Run")
                })
                .map(|a| a.activity.unwrap())
                .collect::<Vec<_>>();

            return Ok(activities);
        } else {
            error!("[JOB-{job_id}][{athlete_id}] No activity data found from athlete");
        }

        Err(error_custom!("No activity data found"))
    }

    async fn parse_activity(activity: &Activity, job_id: i64) -> Result<StravaActivity, Error> {
        let mut repo = REPO.clone();
        let activity_id = activity.id.clone();
        if repo.activity_exists(&activity_id).await? {
            return Err(error_custom!("Activity already exists"));
        }

        let url = format!("https://www.strava.com/activities/{activity_id}/overview");
        let mut client: Client;
        let mut res: reqwest::Response;
        let mut html: String;
        let mut attempt = 1;

        loop {
            client = utils::build_client(None).await;
            res = client.get(url.clone()).send().await?;
            let status = res.status();

            html = res.text().await?;
            let document = Document::from_read(html.as_bytes())?;
            let body = document.find(Name("body")).next().unwrap();
            let inline_stats = body.find(Class("inline-stats")).next();

            if status.is_success() && inline_stats.is_some() {
                break;
            }

            attempt += 1;
            if attempt > 36 {
                error!("[JOB-{job_id}][{activity_id}][SKIP] Failed to fetch activity ID: {status}");
                return Err(error_custom!("Failed to fetch activity"));
            }
        }

        let document = Document::from_read(html.as_bytes())?;
        let body = document.find(Name("body")).next().unwrap();

        info!(
            "[JOB-{job_id}][{activity_id}] from {athlete_name} - {activity_name}",
            activity_id = activity.id,
            athlete_name = activity.athlete.athlete_name,
            activity_name = activity.activity_name.trim()
        );

        let mut stats: HashMap<String, String> = HashMap::new();

        let re = Regex::new(r#"(?s)pageView\.activity\(\)\.set\(\{\s*(?P<content>.*?)\s*\}\);"#)
            .unwrap();

        let raw_stats = if let Some(last) = re.captures_iter(&html).last() {
            let re = Regex::new(r#"(?m)^\s*(\w+)\s*:"#).unwrap();
            let quoted = re.replace_all(&last["content"], r#""$1":"#);

            let content = format!("{{{}}}", quoted);
            serde_json::from_str(&content).expect("Failed to parse raw stats")
        } else {
            RawStats::default()
        };

        let inline_stats = body.find(Class("inline-stats")).next();
        if inline_stats.is_none() {
            error!("[JOB-{job_id}][{activity_id}][SKIP] No inline stats found from activity");
            return Err(error_custom!("No inline stats found"));
        }

        let inline_stats = inline_stats.unwrap();
        for li in inline_stats.find(Name("li")) {
            let label = li.find(Class("label")).next().unwrap();
            let label = label.text().trim().to_case(Case::Snake);
            let value = li.find(Name("strong")).next().unwrap();
            let value = value.text().trim().replace(" ", "").to_string();

            stats.insert(label, value);
        }

        if let Some(more_stats) = body.find(Class("more-stats")).next() {
            for div in more_stats.find(Class("row")) {
                let label = div.find(Class("spans5")).next().unwrap();
                let label = label.text().trim().to_case(Case::Snake);
                let value = div.find(Class("spans3")).next().unwrap();
                let value = value.text().trim().replace(" ", "").to_string();

                stats.insert(label, value);
            }
        }

        let result = StravaActivity {
            activity_id: activity.id.parse().unwrap(),
            strava_id: activity.athlete.athlete_id.to_owned(),
            distance_m: raw_stats.distance.map(|f| f.round() as i32),
            elev_gain_m: raw_stats.elev_gain.map(|f| f.round() as i32),
            moving_time_s: raw_stats.moving_time.map(|f| f as i32),
            elapsed_time_s: stats
                .get("elapsed_time")
                .map(elapsed_time_to_sec)
                .unwrap_or_default(),
            pace_sec_per_km: stats.get("pace").map(pace_to_sec).unwrap_or_default(),
            pace_text: stats.get("pace").map(|s| s.to_string()),
            calories: raw_stats.calories.map(|f| f.round() as f32),
            avg_cadence: raw_stats.avg_cadence.map(|f| f.round() as f32),
            trainer: raw_stats.trainer,
            sport_type: Some(activity.activity_type.to_owned().to_lowercase()),
            athlete_name: Some(activity.athlete.athlete_name.to_owned()),
            payload: serde_json::to_string(&json!({
                "activity": activity,
                "stats": stats,
                "raw_stats": raw_stats
            }))
            .ok(),
            activity_date: activity.start_date.to_owned(),
            scraped_at: Utc::now().naive_utc(),
        };

        Ok(result)
    }
}
