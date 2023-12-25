use std::collections::HashMap;

use crate::Method;

pub trait HttpRequestExtend {
    fn set_remote_addr(&mut self, addr: &str);
    fn get_remote_addr(&self) -> String;
}

// PartialEq:方便在Test中进行对比,否则assert_eq将不能对该类型进行断言对比
#[derive(Debug, PartialEq)]
pub enum Version {
    V1_1,
    V2_0,
    NoSupport,
}

impl From<&str> for Version {
    fn from(version: &str) -> Version {
        match version {
            "HTTP/1.1" => Version::V1_1,
            "HTTP/2.0" => Version::V2_0,
            _ => Version::NoSupport,
        }
    }
}

// 获取 Content-Type 用于解析请求参数
enum ContentType {
    JSON,
    HTML,
    XML,
    TEXT,
    OctetStream,
    FormData,
    XWwwFormUrlencoded,
    NoSupport,
}

impl From<&str> for ContentType {
    fn from(content_type: &str) -> ContentType {
        match content_type {
            "application/json" => ContentType::JSON,
            "text/html" => ContentType::HTML,
            "text/xml" => ContentType::XML,
            "text/plain" => ContentType::TEXT,
            "multipart/form-data" => ContentType::FormData,
            "application/octet-stream" => ContentType::OctetStream,
            "application/x-www-form-urlencoded" => ContentType::XWwwFormUrlencoded,
            _ => ContentType::NoSupport,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Rescues {
    Path(String),
}

impl From<&str> for Rescues {
    fn from(rescues: &str) -> Rescues {
        Rescues::Path(rescues.to_string())
    }
}

#[cfg(test)]
mod test_version {
    use crate::Method;

    use super::*;
    #[test]
    fn test_version() {
        assert_eq!(Version::from("HTTP/1.1"), Version::V1_1);
        assert_eq!(Version::from("HTTP/2.0"), Version::V2_0);
        assert_eq!(Version::from("HTTP/3.0"), Version::NoSupport);
    }

    #[test]
    fn test_method() {
        assert_eq!(Method::from("GET"), Method::GET);
        assert_eq!(Method::from("POST"), Method::POST);
        assert_eq!(Method::from("PUT"), Method::PUT);
        assert_eq!(Method::from("DELETE"), Method::DELETE);
        // assert_eq!(Method::from("TEST"), Method::NoSupport);
    }

    #[test]
    fn test_rescues() {
        assert_eq!(Rescues::from("/"), Rescues::Path("/".to_string()));
        assert_eq!(
            Rescues::from("/index.html"),
            Rescues::Path("/index.html".to_string())
        );
    }
}

#[derive(Debug, PartialEq)]
pub struct HttpRequest<'a> {
    pub(crate) method: Method,
    pub(crate) uri: String,
    pub(crate) version: Version,
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Option<String>,
    pub(crate) more: HashMap<&'a str, String>,
    pub(crate) params: Option<String>,
}

impl<'a> From<String> for HttpRequest<'a> {
    fn from(request: String) -> Self {
        let request_default = HttpRequest::default();
        if request.contains("HTTP/") {
            return HttpRequest::parse_request(&request);
        }
        request_default
    }
}

impl<'a> HttpRequest<'a> {
    fn parse_request(request: &str) -> Self {
        // 空行之后是body内容
        let mut empty_line = false;
        let mut body: Option<String> = None;
        let lines = request.lines().collect::<Vec<&str>>();
        let mut headers: HashMap<String, String> = HashMap::new();
        let mut line_iter = lines.iter();
        let protol_header = line_iter.next().unwrap();
        let mut protol_headers = protol_header.split_whitespace();
        let method = protol_headers.next().unwrap();
        let rescues = protol_headers.next().unwrap();
        let version = protol_headers.next().unwrap();

        let uri = match rescues.split("?").clone().nth(0) {
            Some(s) => s,
            None => "",
        };
        let params = match rescues.split("?").clone().nth(1) {
            Some(s) => s,
            None => "",
        };
        // println!("{uri} {params}");
        // println!("{:?}", line_iter);

        let mut s: Vec<String> = Vec::new();

        for line in line_iter {
            if line.is_empty() || line.len() == 0 {
                empty_line = true;
                continue;
            }
            if empty_line {
                // format!("{}{}", "xxx_", line);
                // s += line.to_string();
                s.push(line.to_string());
                // body = Some(line.to_string());
                continue;
            }
            let mut headers_iter = line.splitn(2, ':');
            let key = headers_iter.next().unwrap();
            let value = headers_iter.next().unwrap();
            headers.insert(key.to_string(), value.trim().to_string());
        }

        if s.len() > 0 {
            body = Some(s.join(""));
        }

        // println!(
        //     "s: {:?}",
        //     format!("method = {method}, uri = {rescues}, version = {version}")
        // );
        // println!("headers: {:?}", headers);
        // println!("body: {:?}", body);

        // line_iter.as_slice().split_inclusive(pred)
        // HttpRequest {
        //     method: method.into(),
        //     uri: rescues.into(),
        //     version: version.into(),
        //     headers: headers,
        //     body,
        //     more: HashMap::new(),
        // }
        HttpRequest {
            method: method.into(),
            uri: uri.to_string(),
            version: version.into(),
            headers: headers,
            body,
            more: HashMap::new(),
            params: Some(params.to_string()),
        }
    }
}

#[warn(dead_code)]
impl<'a> HttpRequest<'a> {
    pub fn get_header(&self, key: &str) -> Option<&str> {
        self.headers.get(key).map(|value| value.as_str())
    }
    pub fn get_uri(&self) -> &str {
        self.uri.as_str()
    }
    pub fn get_header_all(&self) -> &HashMap<String, String> {
        &self.headers
    }
    pub fn get_body(&self) -> Option<&str> {
        self.body.as_ref().map(|value| value.as_str())
    }
    pub fn get_params(&self) -> Option<&str> {
        self.params.as_ref().map(|value| value.as_str())
    }
    pub fn set_remote_addr(&mut self, addr: &str) {
        self.more.insert("remote_addr", addr.to_owned());
    }

    pub fn get_remote_addr(&self) -> String {
        self.more
            .get("remote_addr")
            .unwrap_or(&"".to_string())
            .to_string()
    }

    pub fn get_method(&self) -> Method {
        self.method
    }
}

impl<'a> Default for HttpRequest<'a> {
    fn default() -> Self {
        HttpRequest {
            method: Method::GET,
            uri: "/".to_string(),
            version: Version::V1_1,
            headers: {
                let mut headers: HashMap<String, String> = HashMap::new();
                headers.insert("Content-Type".to_string(), "text/html".to_string());
                headers
            },
            body: None,
            more: HashMap::new(),
            params: Some("".to_string())
        }
    }
}

#[cfg(test)]
mod test_http_request {
    use std::collections::HashMap;

    use crate::{
        Method,
        request::{HttpRequest, Version},
    };

    #[test]
    fn test_parse_request() {
        let request_str = "GET / HTTP/1.1\r\n";
        let request = HttpRequest::from(request_str.to_string());
        assert_eq!(request.method, Method::GET);
        assert_eq!(request.uri, "/".to_string());
        assert_eq!(request.version, Version::V1_1);
        assert_eq!(request.headers, HashMap::new());
        assert_eq!(request.body, None);
    }

    #[test]
    fn test_parse_request_header_and_body() {
        let request_str = "GET / HTTP/1.1\r\nContent-Type: text/html\r\n\r\nbody";
        let request = HttpRequest::from(request_str.to_string());
        let mut header = HashMap::new();
        header.insert("Content-Type".to_string(), "text/html".to_string());
        assert_eq!(request.method, Method::GET);
        assert_eq!(request.uri, "/".to_string());
        assert_eq!(request.version, Version::V1_1);
        assert_eq!(request.headers, header);
        assert_eq!(request.body, Some("body".to_string()));
    }
}
