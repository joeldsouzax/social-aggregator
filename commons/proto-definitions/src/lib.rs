use prost::Message;
use social::v1::Service;

pub mod social {
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/social.v1.rs"));
    }
}

pub mod prost_timestamp_serde;

pub use social::v1;

impl PostId for social::v1::Post {
    fn id(&self) -> String {
        let service_enum = Service::try_from(self.service);
        let service = match service_enum {
            Ok(Service::Mastodon) => "mastodon",
            Ok(Service::X) => "x",
            _ => "unknown",
        };
        format!("{service}:{}", self.id)
    }
}

pub trait PostId: Message {
    fn id(&self) -> String;
}
