#![allow(dead_code)]
use crate::schema::*;
use diesel::mysql::Mysql;
use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(Mysql))]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub strava_id: String,
}

#[derive(Queryable, Debug, Clone)]
#[diesel(table_name = strava_activities_filter)]
#[diesel(check_for_backend(Mysql))]
pub struct StravaActivityFilter {
    pub activity_id: i64,
    pub strava_id: String,
    pub distance_m: Option<i32>,
    pub elev_gain_m: Option<i32>,
    pub moving_time_s: Option<i32>,
    pub elapsed_time_s: Option<i32>,
    pub pace_sec_per_km: Option<i16>,
    pub pace_text: Option<String>,
    pub calories: Option<f64>,
    pub avg_cadence: Option<f64>,
    pub trainer: Option<i8>,
    pub sport_type: Option<String>,
    pub athlete_name: Option<String>,
    pub payload: String,
    pub scraped_at: chrono::NaiveDateTime,
    pub activity_date: Option<chrono::NaiveDateTime>,
}

#[derive(Queryable, Debug, Clone)]
#[diesel(table_name = scrap_logs)]
#[diesel(check_for_backend(Mysql))]
pub struct ScrapLog {
    pub id: i64,
    pub user_id: u64,
    pub strava_id: String,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
}
