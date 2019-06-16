# pshbullet_client

Unofficial [Pushbullet API](https://docs.pushbullet.com/) client.

This crate has support for a limited set of the APIs.

### Push API
* create-push -- except for the file type push
* list-push
* update-push -- not supported
* delete-push -- not supported
* delete-all-pushes -- not supported

### Device API
* list-devices
* create-device -- not supported
* update-device -- not supported
* delete-device -- not supported

Chat, Subscription, User, Upload API are not supported.

## Examples

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

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
