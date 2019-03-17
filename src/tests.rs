#[cfg(test)]
mod tests {
    use crate::rocket;
    use rocket::http::Status;
    use rocket::local::Client;
    use serde_json::from_str;
    #[derive(Serialize, Deserialize, Debug)]
    struct Metadata {
        screen: String,
        description: String,
    }
    fn client() -> Client {
        Client::new(rocket()).expect("valid rocket instance")
    }
    fn token() -> String {
        env!("RAVENSERVER_AT").to_string()
    }
    fn get_metadata(client: &Client, name: &str) -> Metadata {
        from_str(
            client
                .get("/themes/meta/fall")
                .dispatch()
                .body_string()
                .unwrap()
                .as_str(),
        )
        .unwrap()
    }
    #[test]
    fn get_fall_metadata() {
        let client = client();
        let mut response = client.get("/themes/meta/fall").dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
    #[test]
    fn no_token() {
        let client = client();
        let mut response = client
            .post("/themes/meta/fall?typem=screen&value=https://malicious.com")
            .dispatch();
        assert_eq!(response.status(), Status::Unauthorized);
    }
    #[test]
    fn metadata_not_found() {
        let client = client();
        let mut response = client.get("/themes/meta/utter_gibberish").dispatch();
        assert_eq!(response.status(), Status::NotFound);
    }
    #[test]
    fn update_metadata() {
        let client = client();
        let original_metadata = get_metadata(&client, "fall");
        let mut response = client
            .post(format!(
                "/themes/meta/fall?typem=description&value=testing&token={}",
                token()
            ))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
        let new_metadata = get_metadata(&client, "fall");
        assert_ne!(new_metadata.description, original_metadata.description);
        assert_eq!(new_metadata.screen, original_metadata.screen);
        client
            .post(format!(
                "/themes/meta/fall?typem=description&value={}&token={}",
                rocket::http::uri::Uri::percent_encode(original_metadata.description.as_str()),
                token()
            ))
            .dispatch();
    }
}
