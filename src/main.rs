mod tui;

use color_eyre::Result;
use rust_http::{client::HttpClient, echo_server::setup_server};
use tui::App;


fn main() -> Result<()> {
    let server_addr ="127.0.0.1:8004".to_string();
    setup_server(&server_addr)?;

    let client_addr = "127.0.0.1:8005".to_string();
    let client = HttpClient::new(&client_addr)?;

    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new(client, server_addr).run(terminal);
    ratatui::restore();
    app_result
}