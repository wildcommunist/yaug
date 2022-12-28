use crate::domain::login_credentials::email::UserEmail;
use crate::domain::login_credentials::password::LoginPassword;

pub struct LoginCredentials {
    pub email: UserEmail,
    pub password: LoginPassword,
}