mod app;
mod eplot;
mod chaninfo;

use app::MyApp;


#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    let mut options = eframe::NativeOptions::default();
    options.viewport.maximized = Some(true);
    eframe::run_native(
        "EEGUI",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc))))
    )
}
