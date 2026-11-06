//! The app's main thread gets 8 MB of stack, as the game's has on macOS, Linux and Windows
//! (horizon-client's build.rs): iOS gives it 1 MB, and the renderer's start (`Renderer::new`)
//! overflowed it on an iPhone (the Simulator's main thread has 8 MB already).
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("ios") {
        println!("cargo:rustc-link-arg-bin=horizon-ios=-Wl,-stack_size,0x800000");
    }
}
