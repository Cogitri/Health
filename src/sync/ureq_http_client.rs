use http::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use http::method::Method;
use http::status::StatusCode;
use oauth2::{HttpRequest, HttpResponse};

pub fn http_client(request: HttpRequest) -> Result<HttpResponse, ureq::Error> {
    let mut req = if let Method::POST = request.method {
        ureq::post(&request.url.to_string())
    } else {
        ureq::get(&request.url.to_string())
    };

    for (name, value) in request.headers {
        if let Some(name) = name {
            req = req.set(&name.to_string(), value.to_str().unwrap());
        }
    }

    let response = if let Method::POST = request.method {
        req.send(&*request.body)
    } else {
        req.call()
    }?;

    Ok(HttpResponse {
        status_code: StatusCode::from_u16(response.status()).unwrap(),
        headers: vec![(
            CONTENT_TYPE,
            HeaderValue::from_str(response.content_type()).unwrap(),
        )]
        .into_iter()
        .collect::<HeaderMap>(),
        body: response.into_string().unwrap().as_bytes().into(),
    })
}
