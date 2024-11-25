use std::fmt::Display;

use tracing::info;

/// data structure for holding init time (approximate) as well as interval measurement in async context
pub struct Stopwatch {
    original_start: tokio::time::Instant,
    start: tokio::time::Instant,
}

impl Stopwatch {
    /// takes in a displayable and as_ref<str> type as reference to initiate a new stopwatch instance
    pub fn new<T>(message: &T) -> Self
    where
        T: Display + AsRef<str> + ?Sized,
    {
        if !message.as_ref().is_empty() {
            println!(
                "{}  INFO {}: {}",
                chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                env!("CARGO_PKG_NAME"),
                message
            );
        } else {
            println!(
                "{}  INFO {}",
                chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
                env!("CARGO_PKG_NAME"),
            );
        }

        let now = tokio::time::Instant::now();

        Self {
            original_start: now,
            start: now,
        }
    }

    /// displays time since last click or init with the message
    pub fn click<T>(&mut self, message: &T)
    where
        T: Display + ?Sized,
    {
        info!("{}: {:?}", message, self.start.elapsed());
        self.start = tokio::time::Instant::now();
    }

    /// displays time since init with message
    pub fn total<T>(&self, message: &T)
    where
        T: Display + ?Sized,
    {
        info!("{}: {:?}", message, self.original_start.elapsed())
    }

    /// returns original start tokio::time::Instant
    pub fn get_original_start(&self) -> tokio::time::Instant {
        self.original_start
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    /// This test checks if a new instance of Stopwatch is created.
    /// It verifies that the original start time is set to a time earlier
    /// than the current `tokio::time::Instant::now()`, indicating successful initialization.
    #[test]
    fn test_new_creates_instance() {
        let stopwatch = Stopwatch::new(&"Start stopwatch");
        assert!(stopwatch.get_original_start() < tokio::time::Instant::now());
    }

    /// This asynchronous test validates the `click` and `total` methods of Stopwatch.
    /// It first ensures that after calling `click`, the new start time is less than the previous elapsed time.
    /// Then, it verifies that `total` does not affect the elapsed time from original start time.
    #[tokio::test]
    async fn test_click_and_total() {
        let mut stopwatch = Stopwatch::new(&"Starting stopwatch");

        sleep(Duration::from_millis(100));
        let start_elapsed = stopwatch.start.elapsed();

        stopwatch.click(&"Clicked stopwatch");
        assert!(stopwatch.start.elapsed() < start_elapsed);

        sleep(Duration::from_millis(50));
        let click_elapsed = stopwatch.start.elapsed();

        stopwatch.total(&"Total elapsed time");
        assert!(stopwatch.original_start.elapsed() >= click_elapsed);
    }
}
