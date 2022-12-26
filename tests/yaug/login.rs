use crate::helpers::spawn_test_app;

#[tokio::test]
async fn protected_page_redirects_to_login_with_message() {
    let app = spawn_test_app().await;
    todo!()
}

#[tokio::test]
async fn invalid_credentials_redirect_to_login_with_message() {
    todo!()
}

#[tokio::test]
async fn valid_credentials_redirect_to_account_home() {
    todo!()
}
