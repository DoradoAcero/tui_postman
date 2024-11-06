use std::io::Error;

use rust_http::{client::HttpClient, echo_server::setup_server, http::{HttpMethod, HttpRequest}};

fn main() -> Result<(), Error> {
    let server_addr ="127.0.0.1:8004".to_string();
    setup_server(&server_addr)?;

    let client_addr = "127.0.0.1:8005".to_string();
    let client = HttpClient::new(&client_addr)?;
    
    let req = HttpRequest {
        method: HttpMethod::Get,
        endpoint: "localhost:8000".to_string(),
        headers: vec![],
        body: "Hello, ".to_string()
    };

    let res = client.send(req, &server_addr)?;
    println!("{:.?}", res);

    Ok(())
}
