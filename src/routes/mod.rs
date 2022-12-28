mod account;
mod login;
mod home;
mod register;

pub use login::*;
pub use account::get_account_home;
pub use home::get_home_page;
pub use register::{get_register_form, post_register};