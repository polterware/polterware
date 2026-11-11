use polterware_core::local_v1::CheckResponse;

#[test]
fn test_health_types() {
    let response = CheckResponse {
        status: "ok".to_string(),
        version: "0.2.0".to_string(),
        ..CheckResponse::default()
    };
    assert!(!response.status.is_empty());
}
