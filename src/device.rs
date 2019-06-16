//! Pushbullet Device API
//!
//! See [`PushbulletClient::list_devices()`](../struct.PushbulletClient.html#method.list_devices)

use super::*;


lazy_static! {
    static ref DEVICES_URL: String = format!("{}devices", BASE_URL);
}

/// Response type of the device API
#[derive(Deserialize, Debug)]
pub struct Response {
    /// `false` if the item has been deleted
    pub active: bool,
    /// Unique identifier for this object
    pub iden: String,
    /// Creation time in floating point seconds (unix timestamp)
    pub created: f64,
    /// Last modified time in floating point seconds (unix timestamp)
    pub modified: f64,
    /// Icon to use for this device, can be an arbitrary string.
    pub icon: String,

    /// Version of the Pushbullet application installed on the device
    pub app_version: Option<u32>,
    /// Manufacturer of the device
    pub manufacturer: Option<String>,
    /// Model of the device
    pub model: Option<String>,
    /// Name to use when displaying the device
    pub nickname: Option<String>,
    /// Platform-specific push token.
    pub push_token: Option<String>,
}

impl Response {
    /// Get `created` field as DateTime.
    pub fn created_time(&self) -> DateTime<Utc> {
        float_unix_time2date_time(self.created)
    }

    /// Get `modified` field as DateTime.
    pub fn modified_time(&self) -> DateTime<Utc> {
        float_unix_time2date_time(self.modified)
    }
}

#[derive(Deserialize, Debug)]
struct ResponseVec {
    pub devices: Vec<Response>
}

pub type Result = ::std::result::Result<(Vec<Response>, ResponseHeaders), Box<Error>>;

impl PushbulletClient {
    /// Get a list of devices belonging to the current user.
    pub fn list_devices(&self) -> Result {
        match self.get(&*DEVICES_URL) {
            Ok((raw_response, headers)) => {
                let r: ResponseVec = serde_json::from_reader(raw_response)?;
                Ok((r.devices, headers))
                /*
                let mut buf = String::new();
                match raw_response.read_to_string(&mut buf) {
                    Ok(_) => info!("response body: {}", buf),
                    Err(_) => ()
                }
                Ok((ResponseVec { devices: vec!() }, headers))
                */
            }
            Err(e) => Err(e)
        }
    }
}


#[cfg(test)]
mod tests {
    use serde_json;
    use super::*;

    #[test]
    fn deserialize_devices_test() {
        let deserialized: ResponseVec = serde_json::from_str(DEVICES_RESULT).unwrap();
        assert_eq!(deserialized.devices.len(), 2);
        let r = &deserialized.devices[0];
        assert_eq!(r.app_version, Some(8623));
        assert_eq!(r.active, true);
        assert_eq!(r.iden, "ujpah72o0sjAoRtnM0jc");

        assert_eq!(r.modified, 1.412047948579031e+09);
//        println!("{}", r.modified_time().to_rfc3339());
        let diff = r.modified_time().signed_duration_since(Utc.ymd(2014, 9, 30).and_hms(3, 32, 28));
        assert!((diff.num_milliseconds() - 579).abs() < 10);

        assert_eq!(r.created, 1.412047948579029e+09);
//        println!("{}", r.created_time().to_rfc3339());
        let diff = r.created_time().signed_duration_since(Utc.ymd(2014, 9, 30).and_hms(3, 32, 28));
        assert!((diff.num_milliseconds() - 579).abs() < 10);
    }

    const DEVICES_RESULT: &str = r#"
{
  "devices": [
    {
      "active": true,
      "app_version": 8623,
      "created": 1.412047948579029e+09,
      "iden": "ujpah72o0sjAoRtnM0jc",
      "manufacturer": "Apple",
      "model": "iPhone 5s (GSM)",
      "modified": 1.412047948579031e+09,
      "nickname": "Elon Musk's iPhone",
      "push_token": "production:f73be0ee7877c8c7fa69b1468cde764f",
      "type": "ios",
      "kind": "ios",
      "pushable": true,
      "icon": "phone"
    },
    {
      "active": false,
      "iden": "ujCf8vfVeUumdk2AXMrt7Y",
      "created": 1.4369858538733912e+09,
      "modified": 1.445097271901183e+09,
      "pushable": false,
      "icon": "phone"
    }
  ]
}
    "#;
}
