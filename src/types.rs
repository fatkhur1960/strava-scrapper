use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Props {
    pub url: String,
    pub scope: String,
    pub app_context: AppContext,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppContext {
    #[serde(rename = "preFetchedEntries")]
    pub entries: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub entity: String,
    pub activity: Option<Activity>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: String,
    pub activity_name: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub athlete: Athlete,
    pub start_date: String,
    pub start_date_local: Option<String>,
    pub elapsed_time: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Athlete {
    pub athlete_id: String,
    pub avatar_url: String,
    pub athlete_name: String,
    pub sex: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct StravaActivity {
    pub activity_id: i64,
    pub strava_id: String,
    pub distance_m: Option<i32>,
    pub elev_gain_m: Option<i32>,
    pub moving_time_s: Option<i32>,
    pub elapsed_time_s: Option<i32>,
    pub pace_sec_per_km: Option<i16>,
    pub pace_text: Option<String>,
    pub calories: Option<f32>,
    pub avg_cadence: Option<f32>,
    pub trainer: Option<bool>,
    pub sport_type: Option<String>,
    pub athlete_name: Option<String>,
    pub payload: Option<String>,
    pub scraped_at: chrono::NaiveDateTime,
    pub activity_date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct RawStats {
    pub avg_cadence: Option<f64>,
    pub avg_hr: Option<f64>,
    pub avg_speed: Option<f64>,
    pub avg_temp: Option<f64>,
    pub calories: Option<f64>,
    pub distance: Option<f64>,
    pub elev_gain: Option<f64>,
    pub moving_time: Option<i64>,
    pub trainer: Option<bool>,
    pub use_timer_time: Option<bool>,
    pub workout_type: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Proxy {
    pub alive: bool,
    pub proxy: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CookieData {
    pub email: String,
    pub cookie: String,
}
