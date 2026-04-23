use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::osu::{OsuClient, OsuCredentials, SearchResp};
use anyhow::Result;

pub struct OsuClientPool {
    clients: Vec<Arc<Mutex<OsuClient>>>,
    index: AtomicUsize,
}

impl OsuClientPool {
    pub fn new(credentials: Vec<OsuCredentials>) -> Self {
        let clients = credentials
            .into_iter()
            .map(|creds| Arc::new(Mutex::new(OsuClient::new(creds))))
            .collect();
        Self {
            clients,
            index: AtomicUsize::new(0),
        }
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    fn next_client(&self) -> Arc<Mutex<OsuClient>> {
        let idx = self.index.fetch_add(1, Ordering::Relaxed) % self.clients.len();
        self.clients[idx].clone()
    }

    pub async fn search_mania(&self, cursor: Option<&str>, key: u32) -> Result<SearchResp> {
        let client = self.next_client();
        let mut guard: tokio::sync::MutexGuard<'_, OsuClient> = client.lock().await;
        guard.search_mania(cursor, key).await
    }

    pub async fn download_osu_file(&self, beatmap_id: i64) -> Result<Vec<u8>> {
        let client = self.next_client();
        let guard: tokio::sync::MutexGuard<'_, OsuClient> = client.lock().await;
        guard.download_osu_file(beatmap_id).await
    }
}

pub fn load_credentials_from_env() -> Vec<OsuCredentials> {
    let mut creds = Vec::new();

    if let Ok(primary) = OsuCredentials::from_env() {
        creds.push(primary);
    }

    if let (Ok(id), Ok(secret)) = (
        std::env::var("OSU_CLIENT_ID_2"),
        std::env::var("OSU_CLIENT_SECRET_2"),
    ) {
        creds.push(OsuCredentials {
            client_id: id,
            client_secret: secret,
        });
    }

    if let (Ok(id), Ok(secret)) = (
        std::env::var("OSU_CLIENT_ID_3"),
        std::env::var("OSU_CLIENT_SECRET_3"),
    ) {
        creds.push(OsuCredentials {
            client_id: id,
            client_secret: secret,
        });
    }

    creds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_robin_index_wraps() {
        let index = AtomicUsize::new(0);
        let count = 3;

        assert_eq!(index.fetch_add(1, Ordering::Relaxed) % count, 0);
        assert_eq!(index.fetch_add(1, Ordering::Relaxed) % count, 1);
        assert_eq!(index.fetch_add(1, Ordering::Relaxed) % count, 2);
        assert_eq!(index.fetch_add(1, Ordering::Relaxed) % count, 0);
    }
}
