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
async fn malformed_credentials_are_rejected() {
    let app = spawn_test_app().await;

    let test_cases = vec![
        (
            serde_json::json!(
                {
                    "password": Uuid::new_v4().to_string()
                }
            ),
            "missing username"
        ),
        (
            serde_json::json!(
                {
                    "username": Uuid::new_v4().to_string()
                }
            ),
            "missing password"
        ),
        (
            serde_json::json!(
                {
                }
            ),
            "missing username & password"
        ),
    ];

    for (body, message) in test_cases {
        let response = app.post_login(&body).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            "The api did not fail when performing {} test.",
            message
        )
    }
}

#[tokio::test]
async fn valid_credentials_redirect_to_account_home() {
    todo!()
}
