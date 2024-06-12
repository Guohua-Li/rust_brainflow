mod app;
mod eplot;
mod chaninfo;

//use egui::ViewportBuilder;
use app::MyApp;


#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    let mut options = eframe::NativeOptions::default();
    options.viewport.maximized = Some(true);
    /*let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default().with_inner_size([1280.0, 720.0]).with_position([0.0, 0.0]),
        ..Default::default()
    };*/

    eframe::run_native(
        "EEGUI",
        options,
        Box::new(|cc| Box::new(MyApp::new(cc)))
    )
}
