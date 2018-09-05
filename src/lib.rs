#[macro_use]
extern crate cpython;
#[macro_use]
extern crate lando;
extern crate http;
#[macro_use]
extern crate serde_json;

use lando::{LambdaContext, Response, Request, RequestExt};

const HEADER_BLACKLIST: [&'static str; 12] = [
    "x-amz-cf-id",
    "cloudfront-viewer-country",
    "x-forwarded-port",
    "cloudfront-is-smarttv-viewer",
    "x-forwarded-for",
    "cloudfront-is-tablet-viewer",
    "cloudfront-forwarded-proto",
    "x-amzn-trace-id",
    "cloudfront-is-mobile-viewer",
    "cloudfront-is-desktop-viewer",
    "via",
    "x-forwarded-proto",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_filters_headers() {
        let mut request = http::Request::builder();
        request.header("host", "test");
        request.header("x-forwarded-port", "blah");

        let headers = filtered_headers(&request.body("".into()).unwrap()).unwrap();
        assert!(headers.get("x-forwarded-port").is_none());
        assert_eq!(headers.get("host").unwrap(), "test");
    }

    #[test]
    fn it_gives_ip() {
        let request = http::Request::builder().body("".into()).unwrap();
        let ip = ip(&request);
        assert_eq!(ip, "");
    }

    #[test]
    fn it_gives_all() {
        let mut request = http::Request::builder();
        request.header("host", "test");
        request.header("x-forwarded-port", "blah");
        let all_data = all(&request.body("".into()).unwrap()).unwrap();
        assert_eq!(all_data["ip"], "");
        let headers = &all_data["headers"];
        assert!(headers.get("x-forwarded-port").is_none());
        assert_eq!(headers.get("host").unwrap(), "test");
    }
}

gateway!(
    "ip" => ip_req,
    "headers" => headers_req,
    "all" => all_req,
);

fn all_req(request: Request, _context: LambdaContext) -> Result<Response<String>, Box<std::error::Error>> {
    let mut response = Response::builder();
    let response = response.body(all(&request)?.to_string())?;

    Ok(response)
}

fn ip_req(request: Request, _context: LambdaContext) -> Result<Response<String>, Box<std::error::Error>> {
        let mut response = Response::builder();
        response.header("content-type", "text/plain");
        let response = response.body(ip(&request))?;

        Ok(response)
}

fn headers_req(request: Request, _context: LambdaContext) -> Result<Response<String>, Box<std::error::Error>> {
        let mut response = Response::builder();
        let response = response.body(filtered_headers(&request)?.to_string())?;

        Ok(response)
}

fn all(request: &Request) -> Result<serde_json::Value, Box<std::error::Error>> {
    Ok(json!({
        "ip": ip(&request),
        "headers": filtered_headers(&request)?,
    }))
}

fn ip(request: &Request) -> String {
    request.request_context().identity.source_ip
}

fn filtered_headers(request: &Request) -> Result<serde_json::Value, Box<std::error::Error>> {
    let mut headers = serde_json::map::Map::new();
    for header in request.headers() {
        if !HEADER_BLACKLIST.contains(&header.0.as_str()) {
            headers.insert(
                header.0.as_str().to_string(), serde_json::Value::String(header.1.to_str()?.to_string())
            );
        }
    }
    Ok(serde_json::Value::Object(headers))
}
