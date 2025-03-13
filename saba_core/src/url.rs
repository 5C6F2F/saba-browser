use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Url {
    url: String,
    host: String,
    port: String,
    path: String,
    params: String,
}

impl TryFrom<String> for Url {
    type Error = String;
    fn try_from(url: String) -> Result<Self, Self::Error> {
        let mut url = Url::new(url);
        if !url.is_http() {
            return Err("Only HTTP scheme is supported.".to_string());
        }

        url.host = url.extract_host();
        url.port = url.extract_port();
        url.path = url.extract_path();
        url.params = url.extract_params();

        Ok(url)
    }
}

impl TryFrom<&str> for Url {
    type Error = String;
    fn try_from(url: &str) -> Result<Self, Self::Error> {
        Url::try_from(url.to_string())
    }
}

impl Url {
    const HTTP_SCHEME: &'static str = "http://";
    const DEFAULT_PORT: &'static str = "80";

    fn new(url: String) -> Self {
        Url {
            url,
            host: String::new(),
            port: String::new(),
            path: String::new(),
            params: String::new(),
        }
    }

    fn host(&self) -> String {
        self.host.clone()
    }

    fn port(&self) -> String {
        self.port.clone()
    }

    fn path(&self) -> String {
        self.path.clone()
    }

    fn params(&self) -> String {
        self.params.clone()
    }

    fn is_http(&self) -> bool {
        self.url.starts_with(Self::HTTP_SCHEME)
    }

    fn extract_host(&self) -> String {
        let url_parts = self.trim_url();

        if let Some(index) = url_parts[0].find(':') {
            // ポート番号を除去する
            url_parts[0][..index].to_string()
        } else {
            url_parts[0].to_string()
        }
    }

    fn extract_port(&self) -> String {
        let url_parts = self.trim_url();

        if let Some(index) = url_parts[0].find(':') {
            url_parts[0][index + 1..].to_string()
        } else {
            // デフォルトのポート番号は80
            Self::DEFAULT_PORT.to_string()
        }
    }

    fn extract_path(&self) -> String {
        let url_parts = self.trim_url();

        if url_parts.len() < 2 {
            return String::new();
        }

        let path_and_params: Vec<&str> = url_parts[1].splitn(2, "?").collect();
        path_and_params[0].to_string()
    }

    fn extract_params(&self) -> String {
        let url_parts = self.trim_url();

        if url_parts.len() < 2 {
            return String::new();
        }

        let path_and_params: Vec<&str> = url_parts[1].splitn(2, "?").collect();
        if path_and_params.len() < 2 {
            String::new()
        } else {
            path_and_params[1].to_string()
        }
    }

    /// HTTPスキームを削除し、'/'で分割された長さ1か2のベクターを返す。
    fn trim_url(&self) -> Vec<&str> {
        self.url
            .trim_start_matches(Self::HTTP_SCHEME)
            .splitn(2, "/")
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::Url;
    use alloc::string::{String, ToString};

    #[test]
    fn test_host() {
        let url = "http://example.com";
        let expected = Ok(Url {
            url: url.to_string(),
            host: "example.com".to_string(),
            port: "80".to_string(),
            path: String::new(),
            params: String::new(),
        });

        assert_eq!(expected, Url::try_from(url));
    }

    #[test]
    fn test_host_port() {
        let url = "http://example.com:8888";
        let expected = Ok(Url {
            url: url.to_string(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: String::new(),
            params: String::new(),
        });

        assert_eq!(expected, Url::try_from(url));
    }

    #[test]
    fn test_host_port_path() {
        let url = "http://example.com:8888/index.html";
        let expected = Ok(Url {
            url: url.to_string(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
            params: String::new(),
        });

        assert_eq!(expected, Url::try_from(url));
    }

    #[test]
    fn test_host_path() {
        let url = "http://example.com/index.html";
        let expected = Ok(Url {
            url: url.to_string(),
            host: "example.com".to_string(),
            port: "80".to_string(),
            path: "index.html".to_string(),
            params: String::new(),
        });

        assert_eq!(expected, Url::try_from(url));
    }

    #[test]
    fn test_host_port_path_param() {
        let url = "http://example.com:8888/index.html?a=123&b=456";
        let expected = Ok(Url {
            url: url.to_string(),
            host: "example.com".to_string(),
            port: "8888".to_string(),
            path: "index.html".to_string(),
            params: "a=123&b=456".to_string(),
        });

        assert_eq!(expected, Url::try_from(url));
    }

    #[test]
    fn test_no_scheme() {
        let url = "example.com";
        let expected = Err("Only HTTP scheme is supported.".to_string());
        assert_eq!(expected, Url::try_from(url));
    }

    #[test]
    fn test_unsupported_scheme() {
        let url = "https://example.com";
        let expected = Err("Only HTTP scheme is supported.".to_string());
        assert_eq!(expected, Url::try_from(url));
    }
}
