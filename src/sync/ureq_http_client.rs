/* ureq_http_client.rs
 *
 * Copyright 2020-2021 Rasmus Thomsen <oss@cogitri.dev>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program. If not, see <http://www.gnu.org/licenses/>.
 */

use http::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    method::Method,
    status::StatusCode,
};
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
