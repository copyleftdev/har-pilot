use har_pilot::{run, Cli, upload_to_s3};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn test_upload_to_s3_mock() {
    let db_path = "test_results.db";
    let bucket = "mock-bucket";

    // Create a dummy SQLite file
    let mut file = File::create(db_path).await.unwrap();
    file.write_all(b"Dummy SQLite content").await.unwrap();

    // Test the mock upload function
    let result = upload_to_s3(db_path, bucket, true).await;
    assert!(result.is_ok());

    // Clean up
    tokio::fs::remove_file(db_path).await.unwrap();
}

#[tokio::test]
async fn test_run_with_mock_s3() {
    let args = Cli {
        har_file: "tests/test.har".to_string(),
        itercount: 1,
        s3_bucket: Some("mock-bucket".to_string()),
    };

    // Prepare a mock HAR file for testing
    let har_content = r#"
    {
        "log": {
            "entries": [
                {
                    "request": {
                        "method": "GET",
                        "url": "http://example.com",
                        "headers": [],
                        "postData": null
                    },
                    "response": {
                        "status": 200
                    }
                }
            ]
        }
    }
    "#;

    let har_path = "tests/test.har";
    let mut har_file = File::create(har_path).await.unwrap();
    har_file.write_all(har_content.as_bytes()).await.unwrap();

    // Run the main functionality with mock S3 upload
    let result = run(args, true).await;
    assert!(result.is_ok());

    // Clean up
    tokio::fs::remove_file(har_path).await.unwrap();
}
