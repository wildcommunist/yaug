use uuid::Uuid;
use crate::helpers::{assert_is_redirected_to, spawn_test_app};

#[tokio::test]
async fn protected_page_redirects_to_login_with_message() {
    let app = spawn_test_app().await;

    let response = app.get_account_home().await;

    assert_is_redirected_to(&response, "/login");
}

#[tokio::test]
async fn invalid_credentials_redirect_to_login_with_message() {
    let app = spawn_test_app().await;
    let username = Uuid::new_v4().to_string();
    let password = Uuid::new_v4().to_string();
    let invalid_body = serde_json::json!(
        {
            "username":username,
            "password":password
        }
    );
    let response = app.post_login(&invalid_body).await;

    assert_is_redirected_to(&response, "/login");
}

#[tokio::test]
async fn valid_credentials_redirect_to_account_home() {
    todo!()
}
