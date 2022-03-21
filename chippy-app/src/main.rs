use app::TemplateApp;

mod app;

fn main() {
    let app = TemplateApp {
        label: "Hello".to_string(),
        value: 5.0
    };

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
