use std::io::Error;

use rust_http::{endpoints::{get, post}, http::{HttpRequest, HttpResponse}, response_codes::ResponseCode, server::Router};


// this is really stupid, but itsa first pass, ill get something running and during getting it going ill find a way to make it more ergonomic
// https://github.com/tokio-rs/axum, I want to aim for a axum like syntax, or at least something relatively close
// I honestly am probably going to stay away from types, and make it fn pointers, and you input the endpoint/method in the add to router fn
fn process(_: HttpRequest) -> HttpResponse {
    HttpResponse {
        status_code: ResponseCode::OK,
        headers: vec![],
        body: "{\"Hello,\": \" World!\"}".to_string(),
    }
}

fn echo(req: HttpRequest) -> HttpResponse {
    HttpResponse { 
        status_code: ResponseCode::OK,
        headers: req.headers.clone(),
        body: format!("{:.?}", req),
    }
}


pub fn setup_my_server(server_addr: &String) -> Result<(), Error> {
    let router = Router::new(&server_addr)?
        .add_endpoint("/".to_string(), get(process))
        .add_endpoint("/echo".to_string(), post(echo));

    router.server_loop()?; // goes off and spawns a thread running the server in the backgrond
    Ok(())
}