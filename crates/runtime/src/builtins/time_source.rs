use serde::Deserialize;
use serde::Serialize;

use crate::builtins::duration::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub enum TimeSource<F> {
    Ingestion {
        watermark_interval: Duration,
    },
    Event {
        extractor: F,
        watermark_interval: Duration,
        slack: Duration,
    },
}

impl<F> TimeSource<F> {
    pub fn ingestion(watermark_interval: Duration) -> Self {
        Self::Ingestion { watermark_interval }
    }
    pub fn event(extractor: F, watermark_interval: Duration, slack: Duration) -> Self {
        Self::Event {
            extractor,
            watermark_interval,
            slack,
        }
    }
}
