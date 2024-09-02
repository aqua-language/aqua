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

impl<T: std::fmt::Display> std::fmt::Display for TimeSource<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TimeSource::Ingestion { watermark_interval } => {
                write!(
                    f,
                    "Ingestion {{ watermark_interval: {} }}",
                    watermark_interval
                )
            }
            TimeSource::Event {
                extractor,
                watermark_interval,
                slack,
            } => write!(
                f,
                "Event {{ extractor: {}, watermark_interval: {}, slack: {} }}",
                extractor, watermark_interval, slack
            ),
        }
    }
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
