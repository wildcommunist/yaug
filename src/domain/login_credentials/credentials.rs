use crate::domain::login_credentials::email::LoginEmail;
use crate::domain::login_credentials::password::LoginPassword;

pub struct LoginCredentials {
    pub email: LoginEmail,
    pub password: LoginPassword,
}