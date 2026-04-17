use rox::codec::{Decoder, Encoder};
use rox::model::RoxChart;
use rox_formats::rox_native::RoxNativeCodec;
use std::path::PathBuf;

pub struct RoxStore {
    base: PathBuf,
}

impl RoxStore {
    pub fn new(base: impl Into<PathBuf>) -> Self {
        Self { base: base.into() }
    }

    pub fn load(&self, hash: &str) -> Result<Option<RoxChart>, String> {
        let path = self.path(hash);
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        let chart = RoxNativeCodec::decode(&bytes).map_err(|e| e.to_string())?;
        Ok(Some(chart))
    }

    pub fn contains(&self, hash: &str) -> bool {
        self.path(hash).exists()
    }

    pub fn save_if_absent(&self, hash: &str, chart: &RoxChart) -> Result<(), String> {
        self.save_if_absent_with_size(hash, chart).map(|_| ())
    }

    pub fn save_if_absent_with_size(&self, hash: &str, chart: &RoxChart) -> Result<Option<u64>, String> {
        let path = self.path(hash);
        if path.exists() {
            return Ok(None);
        }
        std::fs::create_dir_all(&self.base).map_err(|e| e.to_string())?;
        let bytes = RoxNativeCodec::encode(chart).map_err(|e| e.to_string())?;
        let size = bytes.len() as u64;
        std::fs::write(&path, bytes).map_err(|e| e.to_string())?;
        Ok(Some(size))
    }

    fn path(&self, hash: &str) -> PathBuf {
        self.base.join(format!("{hash}.rox"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rox_formats::auto::auto_decode;

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let store = RoxStore::new(dir.path());
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../metron/assets/test.osu");
        let chart = auto_decode(path).expect("decode test.osu");

        store.save_if_absent("abc123", &chart).expect("save");
        let loaded = store.load("abc123").expect("load").expect("should exist");

        assert_eq!(chart.notes.len(), loaded.notes.len());
    }

    #[test]
    fn test_load_missing_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let store = RoxStore::new(dir.path());
        let result = store.load("nonexistent").expect("no error");
        assert!(result.is_none());
    }

    #[test]
    fn test_save_if_absent_does_not_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let store = RoxStore::new(dir.path());
        let chart = RoxChart::new(4);
        store.save_if_absent("hash1", &chart).unwrap();
        let path = dir.path().join("hash1.rox");
        let size1 = std::fs::metadata(&path).unwrap().len();
        store.save_if_absent("hash1", &chart).unwrap();
        let size2 = std::fs::metadata(&path).unwrap().len();
        assert_eq!(size1, size2);
    }
}
