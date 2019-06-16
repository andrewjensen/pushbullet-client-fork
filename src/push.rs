//! Pushbullet Push API
//!
//! See [`PushbulletClient::create_push()`](../struct.PushbulletClient.html#method.create_push)
//! or [`PushbulletClient::list_push()`](../struct.PushbulletClient.html#method.list_push)

use super::*;
use reqwest::Url;


lazy_static! {
    static ref PUSHES_URL: String = format!("{}pushes", BASE_URL);
}

/// Push target type
#[derive(Debug)]
pub enum Target {
    /// broadcast to all of the user's devices
    Broadcast,
    /// Send the push to a specific device
    Device(String),
    /// Send the push to a email address
    Email(String),
    /// Send the push to all subscribers to a channel
    Channel(String),
    /// Send the push to all users who have granted access to this OAuth iden
    Client(String)
}

/// Request type of the push API
#[derive(Debug)]
pub enum Request<'a> {
    /// Parameters for note type push
    Note {
        /// The note's title.
        title: &'a str,
        /// The note's message.
        body: &'a str,
    },
    /// Parameters for link type push
    Link {
        /// The link's title.
        title: &'a str,
        /// A message associated with the link.
        body: &'a str,
        /// The url to open.
        url: &'a str
    }
}

/// Response type of the push API
#[derive(Deserialize, Debug)]
pub struct Response {
    /// `false` if the item has been deleted
    pub active: bool,
    /// Body of the push, used for all types of pushes
    #[serde(default)]
    pub body: String,
    /// Creation time in floating point seconds (unix timestamp)
    pub created: f64,
    /// Direction the push was sent in, can be "self", "outgoing", or "incoming"
    pub direction: String,
    /// `true` if the push has been dismissed by any device or if any device was active when the push was received
    pub dismissed: bool,
    /// Unique identifier for this object
    pub iden: String,
    /// Last modified time in floating point seconds (unix timestamp)
    pub modified: f64,
    /// Email address of the receiver
    pub receiver_email: String,
    /// Canonical email address of the receiver
    pub receiver_email_normalized: String,
    /// User iden of the receiver
    pub receiver_iden: String,
    /// Email address of the sender
    pub sender_email: String,
    /// Canonical email address of the sender
    pub sender_email_normalized: String,
    /// User iden of the sender
    pub sender_iden: String,
    /// Name of the sender
    pub sender_name: String,
    /// Title of the push, used for all types of pushes
    #[serde(default)]
    pub title: String,
    /// URL field, used for `push_type="link"` pushes
    #[serde(default)]
    pub url: String,
    /// Type of the push, one of "note", "file", "link".
    #[serde(rename = "type")]
    pub push_type: String,
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
    pub pushes: Vec<Response>
}

pub type Result = ::std::result::Result<(Response, ResponseHeaders), Box<Error>>;


/// Parameters for [`PushbulletClient::list_push()`](../struct.PushbulletClient.html#method.list_push)
#[derive(Serialize, Debug)]
pub struct ListCondition {
    /// Don't return deleted pushes
    pub active: bool,
    /// Limit on the number of results returned
    pub limit: u32,
    /// Request pushes modified after this timestamp.
    ///
    /// See [`set_modified_after()`](#method.set_modified_after)
    pub modified_after: Option<f64>,
    /// Cursor for getting multiple pages of pushes
    pub cursor: Option<String>,
}

impl ListCondition {
    pub fn new(limit: u32) -> ListCondition {
        ListCondition {
            active: true,
            limit: limit,
            modified_after: None,
            cursor: None,
        }
    }

    /// Set `modified_after` field with DateTime.
    pub fn set_modified_after(&mut self, t: DateTime<Utc>) {
        self.modified_after = Some(date_time2float_unix_time(t));
    }
}

impl PushbulletClient {
    /// Send a push to a device or another person.
    pub fn create_push(&self, target: &Target, request: Request) -> Result {
        debug!("target: {:?}, request: {:?}", target, request);
        let mut json = match request {
            Request::Note { title, body } => json!({
                "type": "note",
                "title": title,
                "body": body
            }),
            Request::Link { title, body, url } => json!({
                "type": "link",
                "title": title,
                "body": body,
                "url": url
            })
        };
        match *target {
            Target::Broadcast => (),
            Target::Device(ref id) => json["device_iden"] = json!(id),
            Target::Email(ref id) => json["email"] = json!(id),
            Target::Channel(ref id) => json["channel_tag"] = json!(id),
            Target::Client(ref id) => json["client_iden"] = json!(id)
        }
        debug!("json: {}", json);

        match self.post(&*PUSHES_URL, json) {
            Ok((raw_response, headers)) => {
                let r: Response = serde_json::from_reader(raw_response)?;
                Ok((r, headers))
            }
            Err(e) => Err(e)
        }
    }

    /// Request push history.
    pub fn list_push(&self, condition: &ListCondition)
                     -> ::std::result::Result<(Vec<Response>, ResponseHeaders), Box<Error>> {
        debug!("condition: {:?}", condition);
        let mut params = vec![
            ("active", format!("{}", condition.active)),
            ("limit", format!("{}", condition.limit)),
        ];
        condition.modified_after.map(|t| {
            params.push(("modified_after", format!("{:e}", t)))
        });
        condition.cursor.as_ref().map(|s| {
            params.push(("cursor", s.to_string()))
        });
        let url = Url::parse_with_params(&*PUSHES_URL, &params).unwrap().into_string();
        match self.get(&url) {
            Ok((raw_response, headers)) => {
                let r: ResponseVec = serde_json::from_reader(raw_response)?;
                Ok((r.pushes, headers))
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
    fn deserialize_test() {
        let deserialized: ResponseVec = serde_json::from_str(PUSH_RESULT).unwrap();
        assert_eq!(deserialized.pushes.len(), 1);
        let r = &deserialized.pushes[0];
        assert_eq!(r.push_type, "note");
        assert_eq!(r.active, true);
        assert_eq!(r.direction, "self");

        assert_eq!(r.modified, 1.412047948579031e+09);
        //        println!("{}", r.modified_time().to_rfc3339());
        let diff = r.modified_time().signed_duration_since(Utc.ymd(2014, 9, 30).and_hms(3, 32, 28));
        assert!((diff.num_milliseconds() - 579).abs() < 10);

        assert_eq!(r.created, 1.412047948579029e+09);
        //        println!("{}", r.created_time().to_rfc3339());
        let diff = r.created_time().signed_duration_since(Utc.ymd(2014, 9, 30).and_hms(3, 32, 28));
        assert!((diff.num_milliseconds() - 579).abs() < 10);
    }

    const PUSH_RESULT: &str = r#"
{
  "pushes": [
    {
      "active": true,
      "body": "Space Elevator, Mars Hyperloop, Space Model S (Model Space?)",
      "created": 1.412047948579029e+09,
      "direction": "self",
      "dismissed": false,
      "iden": "ujpah72o0sjAoRtnM0jc",
      "modified": 1.412047948579031e+09,
      "receiver_email": "elon@teslamotors.com",
      "receiver_email_normalized": "elon@teslamotors.com",
      "receiver_iden": "ujpah72o0",
      "sender_email": "elon@teslamotors.com",
      "sender_email_normalized": "elon@teslamotors.com",
      "sender_iden": "ujpah72o0",
      "sender_name": "Elon Musk",
      "title": "Space Travel Ideas",
      "type": "note"
    }
  ]
}
    "#;
}
