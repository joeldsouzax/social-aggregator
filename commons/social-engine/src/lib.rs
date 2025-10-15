pub mod engine;
pub mod error;
pub mod queue;

use prost::Message;
use queue::FeederQueue;
use std::fmt::Debug;

pub trait SocialFeeder {
    type Message: Message + Debug;
    fn stream(self, queue: FeederQueue<Self::Message>) -> impl Future<Output = ()>;
}
