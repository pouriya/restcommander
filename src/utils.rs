use std::time::Duration;
// use log::debug;
use sha2::{Digest, Sha512};
use tokio::sync::mpsc::Receiver;
use tokio::time::timeout;

pub async fn maybe_receive<T>(
    channel_receiver: &mut Receiver<T>,
    timeout_in_seconds: u8,
    channel_name: String,
) -> Result<Option<T>, String> {
    // debug!(
    //     "waiting {}s for {} channel",
    //     timeout_in_seconds,
    //     channel_name.clone()
    // );
    match timeout(
        Duration::from_secs(timeout_in_seconds.into()),
        channel_receiver.recv(),
    )
    .await
    {
        Err(_) => Ok(None), // Err(Elapsed)
        Ok(None) => Err(format!("could not receive from {} channel", channel_name)),
        Ok(message_or_none) => Ok(message_or_none),
    }
}

pub fn to_sha512(input: impl AsRef<[u8]> + Clone) -> String {
    let mut hasher = Sha512::new();
    hasher.update(input.clone());
    hex::encode(hasher.finalize())
}
