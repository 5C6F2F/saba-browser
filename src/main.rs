#![no_std]
#![cfg_attr(not(target_os = "linux"), no_main)]

extern crate alloc;

use alloc::string::ToString;
use net_wasabi::http::HttpClient;
use noli::prelude::*;

fn main() {
    let client = HttpClient::new();
    access_to_test_html(&client);
    Api::exit(0);
}

entry_point!(main);

#[allow(dead_code)]
fn access_to_example_com(client: &HttpClient) {
    match client.get("example.com".to_string(), 80, "/".to_string()) {
        Ok(res) => print!("response:\n{:#?}", res),
        Err(e) => print!("error:\n{:#?}", e),
    };
}

#[allow(dead_code)]
fn access_to_test_html(client: &HttpClient) {
    match client.get(
        "host.test".to_string(),
        8000,
        "/pages/test.html".to_string(),
    ) {
        Ok(res) => print!("response:\n{:#?}", res),
        Err(e) => print!("error:\n{:#?}", e),
    }
}
