use std::io::Error;

use rust_http::{endpoints::{get, post}, http::{HttpRequest, HttpResponse}, response_codes::ResponseCode, server::Router};


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