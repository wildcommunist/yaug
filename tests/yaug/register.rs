use anyhow::Context;
use claim::assert_ok;
use fake::Fake;
use fake::faker::internet::en::SafeEmail;
use secrecy::{ExposeSecret, Secret};
use yaug::domain::AccountPassword;
use crate::helpers::spawn_test_app;

#[tokio::test]
async fn register_returns_a_303_on_valid_data() {
    let app = spawn_test_app().await;
    let password = AccountPassword::generate_password(8);
    let email: String = SafeEmail().fake();
    let body = format!(
        "email={}&password={}&password_check={}",
        email,
        password.expose_secret(),
        password.expose_secret(),
    );

    let response = app.post_registration(body).await;
    assert_eq!(response.status().as_u16(), 303);
}

#[tokio::test]
async fn register_returns_a_400_on_invalid_data() {
    let app = spawn_test_app().await;
    let email: String = SafeEmail().fake();
    let password = AccountPassword::generate_password(8);
    let password_new = AccountPassword::generate_password(8);
    let test_cases = vec![
        (
            format!(
                "email={}&password={}&password_check={}",
                "",
                password.expose_secret(),
                password.expose_secret()
            ),
            "missing the email"
        ),
        (
            format!(
                "email={}&password={}&password_check={}",
                email,
                "",
                password.expose_secret()
            ),
            "missing the password"
        ),
        (
            format!(
                "email={}&password={}&password_check={}",
                email,
                password.expose_secret(),
                "",
            ),
            "missing the password_check"
        ),
        (
            format!(
                "email={}&password={}&password_check={}",
                email,
                password.expose_secret(),
                password_new.expose_secret(),
            ),
            "password mismatch"
        ),
        (
            format!(
                "email={}&password={}&password_check={}",
                email,
                "12345678",
                "12345678",
            ),
            "weak password"
        ),
    ];

    for (body, msg) in test_cases {
        let response = app.post_registration(body).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Test did not fail when `{}` was sent to registration endpoint",
            msg
        );
    }
}

#[tokio::test]
async fn register_redirects_with_message_on_success() {
    let app = spawn_test_app().await;
    let password = AccountPassword::generate_password(8);
    let email: String = SafeEmail().fake();
    let body = format!(
        "email={}&password={}&password_check={}",
        email,
        password.expose_secret(),
        password.expose_secret(),
    );

    app.post_registration(body).await;

    let html = app.get_login_page_html().await;
    assert!(html.contains("Your account has been created, please check you email for further instructions."))
}

#[tokio::test]
async fn register_persists_valid_account() {
    let app = spawn_test_app().await;
    let password = AccountPassword::generate_password(8);
    let email: String = SafeEmail().fake();
    let body = format!(
        "email={}&password={}&password_check={}",
        email,
        password.expose_secret(),
        password.expose_secret(),
    );

    app.post_registration(body).await;
    let saved = sqlx::query!(
        r#"
        SELECT user_id, email, password_hash
        FROM accounts
        "#,
    ).fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch test user in `register_persists_valid_account`");

    assert_eq!(email, saved.email);

    let token = sqlx::query!(
        r#"
        SELECT token
        FROM activation_token
        WHERE user_id = $1
        "#,
        saved.user_id
    ).fetch_one(&app.db_pool)
        .await
        .expect("Failed to execute token fetch query in `register_persists_valid_account`");
}

#[tokio::test]
async fn register_send_activation_email() {
    todo!()
}

#[tokio::test]
async fn register_activation_links_activate_account() {
    todo!()
}

#[tokio::test]
async fn register_activated_user_can_login() {
    todo!()
}

#[tokio::test]
async fn register_duplicate_email_are_not_accepted() {
    todo!()
}

#[tokio::test]
async fn register_unactivated_user_cannot_login_get_error_message() {
    todo!()
}

#[tokio::test]
async fn registered_accounts_cannot_register() {
    todo!()
}