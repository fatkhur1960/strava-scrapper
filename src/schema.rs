// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Unsigned<BigInt>,
        name -> Varchar,
        email -> Varchar,
        strava_id -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::table! {
    strava_activities (activity_id) {
        activity_id     -> BigInt,
        strava_id       -> Varchar,
        distance_m      -> Nullable<Integer>,
        elev_gain_m     -> Nullable<Integer>,
        moving_time_s   -> Nullable<Integer>,
        elapsed_time_s  -> Nullable<Integer>,
        pace_sec_per_km -> Nullable<SmallInt>,
        pace_text       -> Nullable<Varchar>,
        calories        -> Nullable<Float>,
        avg_cadence     -> Nullable<Float>,
        trainer         -> Nullable<TinyInt>,
        sport_type      -> Nullable<Varchar>,
        athlete_name    -> Nullable<Varchar>,
        payload         -> Text,
        scraped_at      -> Timestamp,
        activity_date   -> Nullable<Varchar>,
    }
}

diesel::table! {
    scrap_logs (id) {
        id -> Unsigned<BigInt>,
        user_id -> Unsigned<BigInt>,
        strava_id -> Varchar,
        activity_id -> Varchar,
        status -> Varchar,
        created_at -> Timestamp,
    }
}

diesel::allow_tables_to_appear_in_same_query!(users, strava_activities,);
