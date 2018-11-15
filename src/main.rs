extern crate actix_web;
extern crate listenfd;
#[macro_use] extern crate serde_derive;
mod artifact_api;

use listenfd::ListenFd;
use actix_web::{server, App, HttpRequest, HttpResponse, http::ContentEncoding};
use artifact_api::{CardSetApi};

fn index(_req: &HttpRequest) -> HttpResponse {
	let card_list = CardSetApi::new().get_cards().expect("Getting cards failed.");
	let mut response: String = "".to_string();
	for card in card_list {
		if let Some(card_img_url) = card.large_image.default {
			response.push_str(
				&format!("<img src=\"{}\" width=\"20%\" />", &card_img_url)
			);
		}
	}
    HttpResponse::Ok()
        .content_encoding(ContentEncoding::Br)
        .content_type("text/html")
        .body(response)
}

fn main() {
    let mut listenfd = ListenFd::from_env();
    let mut server = server::new(|| {
    	vec![
	        App::new()
	            .prefix("/items")
	            .resource("/", |r| r.f(|r| HttpResponse::Ok())),
    	]
    });

    server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l)
    } else {
        server.bind("127.0.0.1:8080").unwrap()
    };

    server.run();
}