extern crate pshbullet_client;
extern crate chrono;
extern crate dotenv;

use std::env;
use dotenv::dotenv;
use chrono::prelude::*;
use pshbullet_client::*;
use pshbullet_client::push::*;


fn get_config(key: &str) -> String {
    let error = format!("couldn't find required environment variable {}", key);
    env::var(key).expect(&error)
}

fn main() {
    dotenv().ok();
    let access_token = get_config("PUSHBULLET_TOKEN");

    let target = Target::Broadcast;
    //        let target = Target::Device(device_id);
    let note_request = Request::Link {
        title: "link title",
        body: &format!("test push, target: {:?}, at {}", target, Local::now()),
        url: "https://www.rust-lang.org/"
    };
    let client = PushbulletClient::new(access_token);
    let result = client.create_push(&target, note_request);
    match result {
        Ok((responses, headers)) => {
            println!("result: {:?}", responses);
            println!("response_headers:");
            println!("  ratelimit_limit: {}", headers.ratelimit_limit.unwrap_or(0));
            println!("  ratelimit_remaining: {}", headers.ratelimit_remaining.unwrap_or(0));
            println!("  ratelimit_reset (UTC): {}",
                     headers.ratelimit_reset_time().map(|t| t.to_rfc3339())
                         .unwrap_or_else(|| "".to_owned()));
        }
        Err(err) => println!("error: {}", err)
    }

    std::process::exit(0)
}
