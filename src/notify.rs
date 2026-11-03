//! Rider notifications. Production sends an SMS through the gateway; the build used
//! for development and demos logs the message instead, after waiting about as long
//! as the gateway takes to accept a message.

use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Notifier {
    latency: Duration,
}

#[derive(Debug)]
pub struct NotifyError(pub String);

impl Notifier {
    pub fn new(latency: Duration) -> Self {
        Self { latency }
    }

    /// Sends the "new job" text to a rider.
    pub async fn job_offer(&self, phone: &str, rider: &str, reference: &str, dropoff: &str) -> Result<(), NotifyError> {
        if phone.is_empty() {
            return Err(NotifyError(format!("{rider} has no phone number on file")));
        }
        tokio::time::sleep(self.latency).await;
        tracing::info!(to = phone, "SMS: Hi {rider}, new job {reference} for {dropoff}. Open Dispatchly for details.");
        Ok(())
    }
}
