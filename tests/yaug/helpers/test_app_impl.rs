use crate::helpers::test_app::TestApp;

impl TestApp {
    //region Account Home
    pub async fn get_account_home(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/account", &self.address))
            .send()
            .await
            .expect("Failed to execute get account homepage html request")
    }

    pub async fn get_account_home_html(&self) -> String {
        self.get_account_home().await.text().await.unwrap()
    }
    //endregion

    //region Login
    pub async fn get_login_page(&self) -> reqwest::Response {
        self.api_client
            .get(&format!("{}/login", &self.address))
            .send()
            .await
            .expect("Failed to execute request to get login page")
    }

    pub async fn get_login_page_html(&self) -> String {
        self.get_login_page().await.text().await.unwrap()
    }

    //endregion

    pub async fn post_login<Body>(&self, body: &Body) -> reqwest::Response
        where
            Body: serde::Serialize
    {
        self.api_client
            .post(&format!("{}/login", &self.address))
            .form(body)
            .send()
            .await
            .expect("Failed to execute login request")
    }
}