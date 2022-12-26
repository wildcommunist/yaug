pub fn assert_is_redirected_to(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status().as_u16(), 302);
    assert_eq!(response.headers().get("Location").unwrap(), location);
}