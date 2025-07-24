use std::{env, fs, sync::Arc};

use cookie_store::{Cookie, CookieStore};
use regex::Regex;
use reqwest::{
    Client, ClientBuilder, Url,
    header::{self, HeaderMap, HeaderValue},
};
use reqwest_cookie_store::CookieStoreMutex;
use select::{document::Document, predicate::Name};

use crate::{CookieData, Error, Proxy};

/// parse HH:MM:SS to seconds or MM:SS to seconds
/// input: "01:10:10 hours" or "30:00 minutes"
pub fn elapsed_time_to_sec(elapsed_time: &String) -> Option<i32> {
    let re = Regex::new(r#"(?m)^\s*(\d+):(\d+):(\d+)\s*"#).unwrap();
    let re2 = Regex::new(r#"(?m)^\s*(\d+):(\d+)\s*"#).unwrap();

    if let Some(caps) = re.captures(elapsed_time) {
        let hours = caps.get(1).unwrap().as_str().parse::<i32>().unwrap();
        let minutes = caps.get(2).unwrap().as_str().parse::<i32>().unwrap();
        let seconds = caps.get(3).unwrap().as_str().parse::<i32>().unwrap();
        Some((hours * 3600 + minutes * 60 + seconds) as i32)
    } else if let Some(caps) = re2.captures(elapsed_time) {
        let minutes = caps.get(1).unwrap().as_str().parse::<i32>().unwrap();
        let seconds = caps.get(2).unwrap().as_str().parse::<i32>().unwrap();
        Some((minutes * 60 + seconds) as i32)
    } else {
        None
    }
}

pub fn pace_to_sec(pace: &String) -> Option<i16> {
    let re = Regex::new(r#"(?m)^\s*(\d+):(\d+)\s*"#).unwrap();

    if let Some(caps) = re.captures(&pace) {
        let minutes = caps.get(1).unwrap().as_str().parse::<i32>().unwrap();
        let seconds = caps.get(2).unwrap().as_str().parse::<i32>().unwrap();
        Some((minutes * 60 + seconds) as i16)
    } else {
        None
    }
}

pub fn create_cookie_store(idx: Option<usize>) -> Arc<CookieStoreMutex> {
    let mut store = CookieStore::default();
    let cookie_data = get_cookie(idx).expect("Failed to get cookie");

    // debug!("Use cookie from {}", cookie_data.email);

    for pair in cookie_data.cookie.split(';') {
        let trimmed = pair.trim();
        if let Some((name, value)) = trimmed.split_once('=') {
            let cookie_str = format!("{}={}", name.trim(), value.trim());
            if let Ok(cookie) =
                Cookie::parse(&cookie_str, &Url::parse("https://www.strava.com").unwrap())
            {
                store
                    .insert(
                        cookie.into_owned(),
                        &Url::parse("https://www.strava.com").unwrap(),
                    )
                    .unwrap();
            }
        }
    }

    Arc::new(CookieStoreMutex::new(store))
}

pub async fn build_client(idx: Option<usize>) -> Client {
    let cookie_store = create_cookie_store(idx);
    let mut client = ClientBuilder::new()
        .default_headers({
            let mut headers = HeaderMap::new();
            headers.insert(
                header::USER_AGENT,
                HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/237.84.2.178 Safari/537.36"),
            );
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/json"));
            headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
            headers
        })
        .connect_timeout(std::time::Duration::from_secs(15))
        .cookie_provider(Arc::clone(&cookie_store));

    let use_proxy: bool = env::var("USE_PROXY")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap();
    if use_proxy {
        if let Some(mut proxy) = get_proxy() {
            loop {
                let check = reqwest::Client::builder()
                    .proxy(reqwest::Proxy::http(&proxy).unwrap())
                    .connect_timeout(std::time::Duration::from_secs(3))
                    .build()
                    .unwrap()
                    .get("https://www.google.com")
                    .send()
                    .await
                    .ok();

                if check.is_some_and(|res| res.status().is_success()) {
                    break;
                } else {
                    proxy = get_proxy().unwrap();
                }
            }
            client = client.proxy(reqwest::Proxy::http(&proxy).unwrap());
        }
    }

    client.build().expect("Failed to build client")
}

pub async fn check_cookies(athlete_id: &str) -> Result<(), Error> {
    let client = build_client(None).await;
    let res = client
        .get(format!("https://www.strava.com/athletes/{athlete_id}"))
        .send()
        .await?;

    let status = res.status();
    let html = res.text().await?;
    let document = Document::from_read(html.as_bytes())?;
    let body = document.find(Name("body")).next().unwrap();

    if !body.attr("class").unwrap_or_default().contains("logged-in") {
        error!("Cookies expired. Please login again.");

        error!("Error: {status}");
        return Err(error_custom!("Not logged in. Session expired!"));
    }

    Ok(())
}

pub fn get_proxy() -> Option<String> {
    let file = fs::read_to_string("./proxies.json").expect("Unable to read proxy file");
    let proxies: Vec<Proxy> = serde_json::from_str(&file).expect("Unable to parse proxy file");
    let proxies = proxies
        .iter()
        .filter(|p| p.alive)
        .map(|addr| addr.proxy.clone())
        .collect::<Vec<String>>();

    if proxies.is_empty() {
        None
    } else {
        let index = rand::random_range(0..proxies.len());
        Some(proxies[index].clone())
    }
}

pub fn get_cookie(idx: Option<usize>) -> Option<CookieData> {
    let file = fs::read_to_string("./cookies.json").expect("Unable to read cookie file");
    let cookies: Vec<CookieData> =
        serde_json::from_str(&file).expect("Unable to parse cookie file");
    let index = rand::random_range(0..cookies.len());

    cookies.get(idx.unwrap_or(index)).cloned()
}
