// Copyright 2017 yasuhara <yasuhara@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*!
Unofficial [Pushbullet API](https://docs.pushbullet.com/) client.

This crate has support for a limited set of the APIs.

## Push API
* [create-push](struct.PushbulletClient.html#method.create_push) -- except for the file type push
* [list-push](struct.PushbulletClient.html#method.list_push)
* update-push -- not supported
* delete-push -- not supported
* delete-all-pushes -- not supported

## Device API
* [list-devices](struct.PushbulletClient.html#method.list_devices)
* create-device -- not supported
* update-device -- not supported
* delete-device -- not supported

Chat, Subscription, User, Upload API are not supported.

# Examples

```rust
use pshbullet_client::*;
use pshbullet_client::push::*;

fn push() {
    let target = Target::Broadcast;
    //let target = Target::Device("<your_device_iden>");
    let note_request = Request::Note {
        title: "note title",
        body: "test push"
    };
    let client = PushbulletClient::new(String::from("<your_access_token_here>"));
    let (result, headers) = client.create_push(&target, note_request).unwrap();
    println!("result: {:?}", result);
    println!("response headers: {:?}", headers);
}
```

See [examples/](examples) directory for more.
*/

#[macro_use]
extern crate log;
extern crate chrono;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;

pub mod push;
pub mod device;

use std::error::Error;
use std::io;
use std::io::Read;
use chrono::prelude::*;
use serde_json::Value;


const BASE_URL: &str = "https://api.pushbullet.com/v2/";

/// Convert unix timestamp in floating point seconds to `DateTime`
pub fn float_unix_time2date_time(t: f64) -> DateTime<Utc> {
    let nano = t.fract() * 1_000_000_000f64;
    Utc.timestamp(t.trunc() as i64, nano.round() as u32)
}

/// Convert `DateTime` to unix timestamp in floating point seconds
pub fn date_time2float_unix_time(t: DateTime<Utc>) -> f64 {
    let unix_time = t.timestamp() as f64;
    unix_time + (t.timestamp_subsec_nanos() as f64 / 1_000_000_000f64)
}

#[derive(Debug)]
pub struct ResponseHeaders {
    pub ratelimit_limit: Option<i64>,
    pub ratelimit_remaining: Option<i64>,
    pub ratelimit_reset: Option<i64>,
}

impl ResponseHeaders {
    pub fn ratelimit_reset_time(&self) -> Option<DateTime<Utc>> {
        self.ratelimit_reset.map(|sec| Utc.timestamp(sec, 0))
    }
}

/// Pushbullet API Client
#[derive(Debug)]
pub struct PushbulletClient {
    access_token: String
}

impl PushbulletClient {
    /// Initialize with an access token.
    pub fn new(access_token: String) -> PushbulletClient {
        PushbulletClient { access_token }
    }

    fn get(&self, url: &str)
            -> Result<(reqwest::blocking::Response, ResponseHeaders), Box<dyn Error>> {
        debug!("url: {}", url);
        debug!("access_token: {}", self.access_token);

        let client = reqwest::blocking::Client::new();
        let mut response = client.get(url)
            .header("Access-Token", self.access_token.clone())
            .send()?;

        if response.status().is_success() {
            debug!("success status: {}", response.status());
            let raw_headers = response.headers();
            let response_headers = parse_response_headers(raw_headers);
            trace!("response_headers: {:?}", response_headers);
            Ok((response, response_headers))
        } else {
            error!("error status: {:?}", response);
            let mut buf = String::new();
            if response.read_to_string(&mut buf).is_ok() {
                error!("error response body: {}", buf)
            }
            Err(From::from(io::Error::new(io::ErrorKind::Other, "Response has error status")))
        }
    }

    fn post(&self, url: &str, json: Value)
            -> Result<(reqwest::blocking::Response, ResponseHeaders), Box<dyn Error>> {
        debug!("url: {}", url);
        debug!("access_token: {}", self.access_token);

        let client = reqwest::blocking::Client::new();
        let mut response = client.post(url)
            .header("Access-Token", self.access_token.clone())
            .json(&json)
            .send()?;

        if response.status().is_success() {
            debug!("success status: {}", response.status());
            let raw_headers = response.headers();
            let response_headers = parse_response_headers(raw_headers);
            trace!("response_headers: {:?}", response_headers);
            Ok((response, response_headers))
        } else {
            error!("error status: {:?}", response);
            let mut buf = String::new();
            if response.read_to_string(&mut buf).is_ok() {
                error!("error response body: {}", buf)
            }
            Err(From::from(io::Error::new(io::ErrorKind::Other, "Response has error status")))
        }
    }
}

fn parse_response_headers(headers: &reqwest::header::HeaderMap) -> ResponseHeaders {
    let ratelimit_limit = headers
        .get("X-Ratelimit-Limit")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<i64>()
        .unwrap();

    let ratelimit_remaining = headers
        .get("X-Ratelimit-Remaining")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<i64>()
        .unwrap();

    let ratelimit_reset = headers
        .get("X-Ratelimit-Reset")
        .unwrap()
        .to_str()
        .unwrap()
        .parse::<i64>()
        .unwrap();

    ResponseHeaders {
        ratelimit_limit: Some(ratelimit_limit),
        ratelimit_remaining: Some(ratelimit_remaining),
        ratelimit_reset: Some(ratelimit_reset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn response_headers_test() {
        let headers = ResponseHeaders {
            ratelimit_limit: Some(16384),
            ratelimit_remaining: Some(16384),
            ratelimit_reset: Some(1496856653)
        };

        let reset = headers.ratelimit_reset_time().unwrap();
        assert_eq!(reset, Utc.ymd(2017, 6, 7).and_hms(17, 30, 53));
    }

    #[test]
    fn timestamp_conversion_test() {
        let now = Utc::now();
        let f = date_time2float_unix_time(now);
        let r = float_unix_time2date_time(f);
        let diff = r.signed_duration_since(now);
        assert!(diff.num_milliseconds().abs() < 10);
    }
}
