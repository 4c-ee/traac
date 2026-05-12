/// Example: Scrobbling tracks
///
/// Usage:
///   cargo run --example scrobble -- YOUR_API_KEY YOUR_SECRET YOUR_SESSION_KEY

use last_fm_rs::{Client, NowPlaying, Scrobble};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args: Vec<String> = std::env::args().collect();

  if args.len() != 4 {
    eprintln!("Usage: {} API_KEY API_SECRET SESSION_KEY", args[0]);
    eprintln!("\nGet a session key by running: cargo run --example auth");
    std::process::exit(1);
  }

  let api_key = &args[1];
  let api_secret = &args[2];
  let session_key = &args[3];

  println!("Last.fm Scrobbling Example\n");

  let client = Client::new(api_key, api_secret).with_session_key(session_key);

  // Update "Now Playing"
  println!("Updating Now Playing...");
  let now_playing = NowPlaying::new("Kendrick Lamar", "Wesley's Theory")
    .with_album("To Pimp a Butterfly")
    .with_track_number(1)
    .with_duration(287);

  client.update_now_playing(&now_playing).await?;
  println!("✓ Now Playing updated\n");

  // Wait a bit (simulating track playing)
  println!("Waiting 5 seconds (simulating track playback)...");
  tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

  // Submit scrobble
  println!("Submitting scrobble...");
  let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

  let scrobble = Scrobble::new("Kendrick Lamar", "Wesley's Theory", timestamp)
    .with_album("To Pimp a Butterfly")
    .with_track_number(1)
    .with_duration(287);

  let response = client.scrobble(&[scrobble]).await?;

  println!("✓ Scrobble submitted");
  println!("  Accepted: {}", response.scrobbles.attr.accepted);
  println!("  Ignored: {}", response.scrobbles.attr.ignored);

  Ok(())
}
