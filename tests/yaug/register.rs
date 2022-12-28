use crate::helpers::spawn_test_app;

#[tokio::test]
async fn register_returns_a_200_on_valid_data() {
    let app = spawn_test_app().await;
    todo!()
}

#[tokio::test]
async fn valid_details_create_account() {
    todo!()
}

#[tokio::test]
async fn valid_credentials_send_activation_email() {
    todo!()
}

#[tokio::test]
async fn activation_links_activate_account() {
    todo!()
}

#[tokio::test]
async fn activated_user_can_login() {
    todo!()
}

#[tokio::test]
async fn duplicate_email_are_not_accepted() {
    todo!()
}

#[tokio::test]
async fn unactivated_user_cannot_login_get_error_message() {
    todo!()
}

#[tokio::test]
async fn registered_accounts_cannot_register() {
    todo!()
}