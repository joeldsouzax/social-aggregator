pub mod builder;
pub mod queue;

use std::fmt::Debug;

use prost::Message;
use queue::FeederQueue;

pub trait SocialFeeder {
    type Message: Message + Debug;
    fn stream(self, queue: FeederQueue<Self::Message>) -> impl Future<Output = ()>;
}

pub trait PostId {
    fn id(&self) -> String;
}
