use std::io::Error;

use rust_http::{http::{HttpMethod, HttpRequest, HttpResponse}, response_codes::ResponseCode, server::{Endpoint, Router}};


// this is really stupid, but itsa first pass, ill get something running and during getting it going ill find a way to make it more ergonomic
// https://github.com/tokio-rs/axum, I want to aim for a axum like syntax, or at least something relatively close
// I honestly am probably going to stay away from types, and make it fn pointers, and you input the endpoint/method in the add to router fn
struct FirstEndpoint {}

impl FirstEndpoint {
    pub fn new() -> FirstEndpoint {
        FirstEndpoint {}
    }
}

impl Endpoint for FirstEndpoint {
    fn process(&self, _: HttpRequest) -> HttpResponse {
        HttpResponse {
            status_code: ResponseCode::OK,
            headers: vec![],
            body: "{\"Hello,\": \" World!\"}".to_string(),
        }
    }

    fn get_endpoint_type(&self) -> HttpMethod {
        HttpMethod::Get
    }
}

pub fn setup_my_server(server_addr: &String) -> Result<(), Error> {
    let mut router = Router::new(&server_addr)?;

    let endpoint = Box::new(FirstEndpoint::new());
    router.add_endpoint("/".to_string(), endpoint);
    
    router.server_loop()?;
    Ok(())
}