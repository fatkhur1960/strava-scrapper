#![allow(dead_code)]
use diesel::prelude::*;

use crate::{Error, StravaActivity, database::*, models::User};

#[derive(Insertable)]
#[diesel(table_name = crate::schema::strava_activities)]
pub struct CreateActivity {
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
    pub trainer: Option<i8>,
    pub sport_type: Option<String>,
    pub athlete_name: Option<String>,
    pub payload: String,
    pub scraped_at: chrono::NaiveDateTime,
    pub activity_date: Option<String>,
}

#[derive(Clone)]
pub struct Repository {
    conn: DbConnMan,
}

impl Repository {
    pub fn new(conn: DbConnMan) -> Self {
        Self { conn }
    }

    pub async fn conn(&mut self) -> DbConn {
        self.conn.get().expect("Failed to get connection")
    }

    pub async fn get_users(
        &mut self,
        limit: i64,
        offset: i64,
    ) -> Result<(Vec<User>, i64), diesel::result::Error> {
        use crate::schema::users;
        let conn = &mut self.conn().await;

        let users = users::table
            .filter(users::strava_id.ne("-").and(users::strava_id.ne("")))
            .select(User::as_select())
            .limit(limit)
            .offset(offset)
            .order_by(users::created_at.desc())
            .load(conn)?;
        let total = users::table
            .filter(users::strava_id.ne("-").and(users::strava_id.ne("")))
            .count()
            .get_result::<i64>(conn)?;

        Ok((users, total))
    }

    pub async fn get_total_users(&mut self) -> Result<i64, diesel::result::Error> {
        use crate::schema::users;
        let conn = &mut self.conn().await;

        Ok(users::table.count().get_result(conn)?)
    }

    pub async fn activity_exists(&mut self, activity_id: &str) -> Result<bool, Error> {
        use crate::schema::strava_activities as activities;
        let conn = &mut self.conn().await;

        let activity_id = activity_id.parse::<i64>().unwrap_or_default();
        let exist = activities::table
            .select(activities::activity_id)
            .filter(activities::activity_id.eq(activity_id))
            .count()
            .get_result::<i64>(conn)?;

        Ok(exist > 0)
    }

    pub async fn create_activities(
        &mut self,
        activities: Vec<StravaActivity>,
    ) -> Result<usize, Error> {
        use crate::schema::strava_activities as activities;
        let conn = &mut self.conn().await;

        let res = diesel::insert_or_ignore_into(activities::table)
            .values(
                activities
                    .iter()
                    .map(|a| CreateActivity {
                        activity_id: a.activity_id,
                        strava_id: a.strava_id.to_owned(),
                        distance_m: a.distance_m,
                        elev_gain_m: a.elev_gain_m,
                        moving_time_s: a.moving_time_s,
                        elapsed_time_s: a.elapsed_time_s,
                        pace_sec_per_km: a.pace_sec_per_km,
                        pace_text: a.pace_text.to_owned(),
                        calories: a.calories,
                        avg_cadence: a.avg_cadence,
                        trainer: a.trainer.map(|a| a.into()),
                        sport_type: a.sport_type.to_owned(),
                        athlete_name: a.athlete_name.to_owned(),
                        payload: a.payload.clone().unwrap_or_default(),
                        activity_date: Some(a.activity_date.to_owned()),
                        scraped_at: a.scraped_at.into(),
                    })
                    .collect::<Vec<_>>(),
            )
            .execute(conn)?;

        Ok(res)
    }
}
