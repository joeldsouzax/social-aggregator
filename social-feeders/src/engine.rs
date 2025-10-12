pub trait SocialFeeder {
    async fn stream(self);
}

pub struct Engine;
