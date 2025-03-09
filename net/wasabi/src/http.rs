use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use noli::net::{lookup_host, IpV4Addr, SocketAddr, TcpStream};
use saba_core::{error::Error, http::HttpResponse};

pub struct HttpClient {}

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, host: String, port: u16, path: String) -> Result<HttpResponse, Error> {
        let ip_addr = self.get_ip_addresses(&host)?;
        let socket_addr: SocketAddr = (ip_addr, port).into();
        let mut stream = match TcpStream::connect(socket_addr) {
            Ok(stream) => stream,
            Err(e) => {
                return Err(Error::Network(format!(
                    "Failed to connect to TCP stream: {:#?}",
                    e
                )))
            }
        };

        let request = self.get_request_message(&host, &path);

        let _bytes_written = match stream.write(request.as_bytes()) {
            Ok(bytes) => bytes,
            Err(e) => {
                return Err(Error::Network(format!(
                    "Failed to send a request to TCP stream: {:#?}",
                    e
                )));
            }
        };

        let mut received = Vec::new();
        loop {
            let mut buf = [0; 4096];
            let bytes_read = match stream.read(&mut buf) {
                Ok(bytes) => bytes,
                Err(e) => {
                    return Err(Error::Network(format!(
                        "Failed to receive a request from TCP stream: {:#?}",
                        e
                    )));
                }
            };

            if bytes_read == 0 {
                break;
            }

            received.extend(&buf[..bytes_read]);
        }

        match core::str::from_utf8(&received) {
            Ok(response) => HttpResponse::try_from(response.to_string()),
            Err(e) => Err(Error::Network(format!(
                "Invalid received response: {:#?}",
                e
            ))),
        }
    }

    fn get_ip_addresses(&self, host: &str) -> Result<IpV4Addr, Error> {
        let ip_addresses = match lookup_host(host) {
            Ok(ip_addresses) => ip_addresses,
            Err(e) => {
                return Err(Error::Network(format!(
                    "Failed to find IP addresses: {:#?}",
                    e
                )))
            }
        };

        if let Some(&ip_address) = ip_addresses.get(0) {
            Ok(ip_address)
        } else {
            Err(Error::Network("Failed to find IP addresses".to_string()))
        }
    }

    fn get_request_message(&self, host: &str, path: &str) -> String {
        format!("GET /{path} HTTP/1.1\nHost: {host}\nAccept: text/html\nConnection: close\n\n")
    }
}
