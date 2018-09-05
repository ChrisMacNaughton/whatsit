#[macro_use]
extern crate cpython;
#[macro_use]
extern crate lando;

use lando::{Response, RequestExt};

gateway!(
    "hello" => |request, _| {
        println!("{:?}", request);
        Ok(lando::Response::new(format!(
            "hello {}",
            request
                .path_parameters()
                .get("name")
                .cloned()
                .unwrap_or_else(|| "stranger".to_owned())
        )))
    },
    "ip" => |request, _context| {
        let mut response = Response::builder();
        response.header("content-type", "text/plain");
        let response = response.body(request.request_context().identity.source_ip)?;

        Ok(response)
    },
);
