use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::time::{Duration, Instant};

const TOKEN_URL: &str = "https://osu.ppy.sh/oauth/token";
const SEARCH_URL: &str = "https://osu.ppy.sh/api/v2/beatmapsets/search";
const OSU_FILE_URL: &str = "https://osu.ppy.sh/osu";
const USER_AGENT: &str = "osef-bridge/0.1";

pub struct OsuClient {
    http: reqwest::Client,
    client_id: String,
    client_secret: String,
    token: Option<String>,
    token_expires: Option<Instant>,
}

#[derive(Deserialize)]
struct TokenResp {
    access_token: String,
    expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct SearchResp {
    pub beatmapsets: Vec<Beatmapset>,
    pub total: Option<u64>,
    pub cursor_string: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Beatmapset {
    pub id: i64,
    pub artist: String,
    pub artist_unicode: Option<String>,
    pub title: String,
    pub title_unicode: Option<String>,
    pub creator: String,
    pub source: Option<String>,
    #[serde(default)]
    pub tags: String,
    #[serde(default)]
    pub video: bool,
    #[serde(default)]
    pub storyboard: bool,
    #[serde(default)]
    pub nsfw: bool,
    pub preview_url: Option<String>,
    pub covers: Option<Covers>,
    pub status: Option<String>,
    pub last_updated: Option<String>,
    pub beatmaps: Option<Vec<Beatmap>>,
}

#[derive(Debug, Deserialize)]
pub struct Covers {
    pub cover: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Beatmap {
    pub id: i64,
    pub beatmapset_id: i64,
    pub version: String,
    pub checksum: Option<String>,
    pub mode_int: i16,
    pub count_circles: i32,
    pub count_sliders: i32,
    pub count_spinners: i32,
    pub max_combo: Option<i32>,
    pub hit_length: i32,
    pub total_length: i32,
    pub bpm: Option<f64>,
    pub cs: f64,
    pub ar: f64,
    pub accuracy: f64,
    pub drain: f64,
    pub status: String,
}

#[derive(Clone, Debug)]
pub struct OsuCredentials {
    pub client_id: String,
    pub client_secret: String,
}

impl OsuCredentials {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            client_id: std::env::var("OSU_CLIENT_ID").context("OSU_CLIENT_ID missing")?,
            client_secret: std::env::var("OSU_CLIENT_SECRET").context("OSU_CLIENT_SECRET missing")?,
        })
    }
}

impl OsuClient {
    pub fn new(creds: OsuCredentials) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
            client_id: creds.client_id,
            client_secret: creds.client_secret,
            token: None,
            token_expires: None,
        }
    }

    async fn fetch_new_token(&self) -> Result<TokenResp> {
        let body = serde_json::json!({
            "client_id": self.client_id,
            "client_secret": self.client_secret,
            "grant_type": "client_credentials",
            "scope": "public",
        });
        self.http
            .post(TOKEN_URL)
            .json(&body)
            .send()
            .await
            .context("token request")?
            .error_for_status()
            .context("token status")?
            .json()
            .await
            .context("token json")
    }

    async fn ensure_token(&mut self) -> Result<String> {
        let needs_refresh = self.token_expires.map_or(true, |expiry| Instant::now() >= expiry);
        if needs_refresh {
            let resp = self.fetch_new_token().await?;
            self.token_expires = Some(
                Instant::now() + Duration::from_secs(resp.expires_in.saturating_sub(60)),
            );
            self.token = Some(resp.access_token);
        }
        self.token.clone().ok_or_else(|| anyhow!("no token after refresh"))
    }

    pub async fn download_osu_file(&self, beatmap_id: i64) -> Result<Vec<u8>> {
        let url = format!("{OSU_FILE_URL}/{beatmap_id}");
        let resp = self.http.get(&url).send().await.context("osu file send")?;
        let status = resp.status();
        if !status.is_success() {
            return Err(anyhow!("osu file {}: {}", status, beatmap_id));
        }
        resp.bytes().await.context("osu file bytes").map(|b| b.to_vec())
    }

    pub async fn search_mania(&mut self, cursor: Option<&str>) -> Result<SearchResp> {
        let token = self.ensure_token().await?;
        let mut req = self
            .http
            .get(SEARCH_URL)
            .bearer_auth(token)
            .query(&[("m", "3"), ("s", "any"), ("nsfw", "true"), ("key", "4")]);
        if let Some(cursor_string) = cursor {
            req = req.query(&[("cursor_string", cursor_string)]);
        }
        let resp = req.send().await.context("search send")?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("search {}: {}", status, body));
        }
        resp.json::<SearchResp>().await.context("search json")
    }
}
