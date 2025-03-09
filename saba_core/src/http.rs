use crate::error::Error;
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

#[derive(Debug)]
pub struct HttpResponse {
    version: String,
    status_code: u32,
    reason: String,
    headers: Vec<Header>,
    body: String,
}

impl TryFrom<String> for HttpResponse {
    type Error = Error;
    fn try_from(raw_response: String) -> Result<Self, Self::Error> {
        let preprocessed_response = raw_response.trim_start().replace("\n\r", "\n");

        let Some((status_line, remaining)) = preprocessed_response.split_once('\n') else {
            return Err(Error::Network(format!(
                "invalid http response: {}",
                preprocessed_response
            )));
        };

        let statuses: Vec<&str> = status_line.split(' ').collect();

        let (headers, body) = match remaining.split_once("\n\n") {
            Some((headers_str, body)) => {
                let mut headers = Vec::new();

                for header in headers_str.split('\n') {
                    let Some((name, value)) = header.split_once(':') else {
                        return Err(Error::Network(format!(
                            "invalid http response: {}",
                            preprocessed_response
                        )));
                    };

                    headers.push(Header::new(name.to_string(), value.to_string()));
                }
                (headers, body)
            }
            None => (Vec::new(), remaining),
        };

        Ok(Self {
            version: statuses[0].to_string(),
            status_code: statuses[1].parse().unwrap_or(404),
            reason: statuses[2].to_string(),
            headers,
            body: body.to_string(),
        })
    }
}

impl HttpResponse {
    pub fn version(&self) -> String {
        self.version.clone()
    }

    pub fn status_code(&self) -> u32 {
        self.status_code
    }

    pub fn reason(&self) -> String {
        self.reason.clone()
    }

    pub fn headers(&self) -> Vec<Header> {
        self.headers.clone()
    }

    pub fn header_value(&self, name: &str) -> Result<String, String> {
        self.headers
            .iter()
            .find(|header| header.name == name)
            .map_or(
                Err(format!("failed to fin {} in headers", name)),
                |header| Ok(header.value.clone()),
            )
    }

    pub fn body(&self) -> String {
        self.body.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Header {
    name: String,
    value: String,
}

impl Header {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

#[cfg(test)]
mod tests {
    use super::HttpResponse;
    use alloc::string::ToString;

    #[test]
    fn test_status_line_only() {
        let raw = "HTTP/1.1 200 OK\n\n".to_string();
        let response = HttpResponse::try_from(raw).expect("failed to parse http response");
        assert_eq!(response.version(), "HTTP/1.1");
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.reason(), "OK");
    }

    #[test]
    fn test_one_header() {
        let raw = "HTTP/1.1 200 OK\nDate:xx xx xx\n\n".to_string();
        let response = HttpResponse::try_from(raw).expect("failed to parse http response");
        assert_eq!(response.version(), "HTTP/1.1");
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.reason(), "OK");

        assert_eq!(response.header_value("Date"), Ok("xx xx xx".to_string()));
    }

    #[test]
    fn test_two_header() {
        let raw = "HTTP/1.1 200 OK\nDate:xx xx xx\nContent-Length:42\n\n".to_string();
        let response = HttpResponse::try_from(raw).expect("failed to parse http response");
        assert_eq!(response.version(), "HTTP/1.1");
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.reason(), "OK");

        assert_eq!(response.header_value("Date"), Ok("xx xx xx".to_string()));
        assert_eq!(
            response.header_value("Content-Length"),
            Ok("42".to_string())
        );
    }

    #[test]
    fn test_body() {
        let raw = "HTTP/1.1 200 OK\nDate:xx xx xx\nContent-Length:42\n\nbody message".to_string();
        let response = HttpResponse::try_from(raw).expect("failed to parse http response");
        assert_eq!(response.version(), "HTTP/1.1");
        assert_eq!(response.status_code(), 200);
        assert_eq!(response.reason(), "OK");

        assert_eq!(response.header_value("Date"), Ok("xx xx xx".to_string()));
        assert_eq!(
            response.header_value("Content-Length"),
            Ok("42".to_string())
        );

        assert_eq!(response.body(), "body message".to_string());
    }

    #[test]
    fn test_invalid() {
        let raw = "HTTP/1.1 200 OK".to_string();
        assert!(HttpResponse::try_from(raw).is_err());
    }

    #[test]
    fn test_invalid_header() {
        let raw = "HTTP/1.1 200 OK\nInvalid Header\n\n".to_string();
        assert!(HttpResponse::try_from(raw).is_err());
    }
}
