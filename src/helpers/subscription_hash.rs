use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

pub fn generate_subscription_token(len: usize) -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(len)
        .collect()
}