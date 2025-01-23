pub mod tests {
    use reqwest::StatusCode;

    #[tokio::test]
    pub async fn test_fail_without_address() {
        let endpoint = format!("http://0.0.0.0:8080/defi/rewards");
        let client = reqwest::Client::new();
        let response = client.get(endpoint).send().await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    pub async fn test_fail_with_invalid_address_format() {
        let address = "0x03fbb5d22e1393e47ff967u88urui3u4iyr3ui4r90sduw0943jowefwruwerowu";
        let endpoint = format!("http://0.0.0.0:8080/defi/rewards?addr={}", address);
        let client = reqwest::Client::new();
        let response = client.get(endpoint).send().await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    pub async fn test_ok_with_valid_address_format() {
        let address = "0x03fbb5d22e1393e47ff9678d12748885f176d8ce96051f72819cd2a6fa062589";
        let endpoint = format!("http://0.0.0.0:8080/defi/rewards?addr={}", address);
        let client = reqwest::Client::new();
        let response = client.get(endpoint).send().await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // #[tokio::test]
    // pub async fn test_get_quest_participants() {
    //     let endpoint = format!("http://
}
