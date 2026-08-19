use whatsapp_api_web::App;

fn main() {
    dioxus_logger::init(tracing::Level::INFO).expect("failed to init logger");
    dioxus::launch(App);
}