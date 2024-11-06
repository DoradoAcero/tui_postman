use rust_http::{echo_server::setup_server, http::{HttpHeader, HttpMethod, HttpRequest, HttpResponse}, response_codes::ResponseCode};

fn main() -> Result<()> {
    let server_addr ="127.0.0.1:8004".to_string();
    let client_addr = "127.0.0.1:8005".to_string();
    let socket = TcpPort::new(&client_addr).expect("couldn't bind to client address");
    
    let req = HttpRequest {
        method: HttpMethod::Get,
        endpoint: "localhost:8000".to_string(),
        headers: vec![],
        body: "Hello, ".to_string()
    };

    let pwost = HttpRequest {
        method: HttpMethod::Post,
        endpoint: "notice this doesn't do jack yet".to_string(),
        headers: vec![HttpHeader{ key: "host".to_string(), value: "localhost:8000".to_string() }],
        body: "Hello, ".to_string()
    };

    assert!(req == HttpRequest::from_string(req.to_string()).unwrap());

    let res = HttpResponse {
        status_code: ResponseCode::OK,
        headers: vec![],
        body: "World!".to_string(),
    };
    assert!(res == HttpResponse::from_string(res.to_string()).unwrap());

    setup_server(&server_addr)?;

    socket.send(req.to_string(), &server_addr)?;
    let (response, _) = socket.recieve()?;
    println!("{:.?}", HttpResponse::from_string(response)?);


    socket.send(pwost.to_string(), &server_addr)?;
    let (response, _) = socket.recieve()?;
    println!("{:.?}", HttpResponse::from_string(response)?);

    Ok(())
}
