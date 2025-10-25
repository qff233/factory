slint::include_modules!();

mod constant;
mod http_client;
mod model;
mod ui;

#[tokio::main]
async fn main() {
    let app = ui::App::new();
    app.run();
}
