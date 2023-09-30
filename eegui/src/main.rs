mod app;
mod eplot;

use app::MyApp; // use crate::app::MyApp;


#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "EEGUI",
        eframe::NativeOptions::default(),
        Box::new(|cc| Box::new(MyApp::new(cc)))
    )
}
