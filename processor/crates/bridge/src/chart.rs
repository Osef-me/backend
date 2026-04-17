use rox::model::RoxChart;
use rox_analysis::hash::normalized_notes_hash;
use rox_formats::{OsuDecodeOptions, OsuDecoder};

pub fn decode_bytes(path_hint: &str, bytes: &[u8]) -> Result<RoxChart, String> {
    if path_hint.ends_with(".osu") {
        OsuDecoder::decode_with_options(bytes, &OsuDecodeOptions { re_arrange_bpm: true })
            .map_err(|e| e.to_string())
    } else {
        rox_formats::auto::auto_decode_bytes(path_hint, bytes).map_err(|e| e.to_string())
    }
}

/// Hash normalized at 100 bpm — canonical dedup key across rate-scaled variants.
pub fn normalized_hash_100(chart: &RoxChart) -> String {
    normalized_notes_hash(chart, 100.0)
}
