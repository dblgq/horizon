//! Horizon on iOS. An iOS app gets no command line and no working directory of
//! its own choosing, so this program points the game at the app's folders, then
//! starts it exactly as `horizon-client` does.
//!
//! - `Documents/` is the game folder (saves, options.txt, resource packs) and the
//!   game's home (logs, crash reports). The Files app shows it, so the prepared
//!   resources go in as `Documents/.local/horizon-client/<version>/` and logs
//!   come out without a cable.
//!
//! winit runs the app's UIApplicationMain on this thread, so `main` never
//! returns while the app lives.
#![forbid(unsafe_code)]

#[cfg(target_os = "ios")]
#[global_allocator]
static ALLOCATOR: horizon_threads::Allocator = horizon_threads::ALLOCATOR;

/// The mod API level this game serves, as in `horizon-client`'s main.rs.
#[cfg(target_os = "ios")]
static MOD_API_LEVEL: [u8; 27] = horizon_mods::api_level::MARK;

#[cfg(target_os = "ios")]
fn main() {
    std::hint::black_box(&MOD_API_LEVEL);
    // iOS sets HOME to the app's own container.
    let container = std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_default();
    let documents = container.join("Documents");
    if let Err(e) =
        std::fs::create_dir_all(&documents).and_then(|()| std::env::set_current_dir(&documents))
    {
        eprintln!("Horizon: game folder {}: {e}", documents.display());
    }
    horizon_client::logging::set_game_home(documents);
    horizon_client::entry::main();
}

#[cfg(not(target_os = "ios"))]
fn main() {
    eprintln!("horizon-ios is the iOS app; elsewhere run horizon-client.");
}
