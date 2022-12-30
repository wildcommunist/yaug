use crate::domain::account_credentials::email::AccountEmail;
use crate::domain::account_credentials::password::AccountPassword;

pub struct AccountCredentials {
    pub email: AccountEmail,
    pub password: AccountPassword,
}