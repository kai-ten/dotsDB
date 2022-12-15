mod book_reviews;

use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
// use arrow2::io::json::read;
use async_once::AsyncOnce;
use aws_config::SdkConfig;
use aws_sdk_s3::Client;
use aws_sdk_s3::types::ByteStream;
use lambda_http::aws_lambda_events::event;
use log::error;
// use crate::book_reviews::{BookReview};

#[macro_use]
extern crate lazy_static;

// HANDLE THE DATA INGESTION
// 1. Read the incoming JSON (this will be a book reviews schema)
// 2. Convert the incoming JSON to Parquet
// 3. Write parquet file to the s3://dotsdb-lakehouse-data/books/data folder

// TELL ICEBERG THAT DATA WAS INSERTED PER SPEC - https://iceberg.apache.org/spec/#specification
// 1. Create a manifest file that references the newly created parquet file
// 2. Create a snapshot of the manifest file in the metadata and update the metadata.json
//      - Must also update Glue with the new metadata.json file when this gets updated
// 3. Write the manifest.avro to the s3://dotsdb-lakehouse-data/books/metadata folder
// 4. Write the metadata.json to the s3://dotsdb-lakehouse-data/books/metadata folder

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples


lazy_static! (
    static ref AWS_CONFIG: AsyncOnce<SdkConfig> = AsyncOnce::new(async { aws_config::load_from_env().await });
    static ref S3_CLIENT: AsyncOnce<aws_sdk_s3::Client> = AsyncOnce::new(async { aws_sdk_s3::Client::new(AWS_CONFIG.get().await) });
);

// fn json_to_parquet(raw_json: &str) {
//     let json = read::json_deserializer::parse(&raw_json)?;
//     let data_type = read::infer(&json)?;
//     let ds = read::deserialize(&json, data_type);
//     println("{:#?}", ds);
//
// }

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    // Extract some useful information from the request

    println!("event - {:?}", event);

    // let body = event.payload::<BookReview>()?;
    // let stream = ByteStream::new(SdkBody::from("hello! This is some data"));

    // TODO: let bucket_name = std::env::var("BUCKET_NAME")
    //     .expect("Set a bucket name in the environment variables.");

    let bucket_name = "";
    let s3_client: &Client = S3_CLIENT.get().await;

    // TODO: should have 3 different s3 writes - data, manifest.avro, schema.json
    let _ = s3_client.put_object()
        .bucket(bucket_name)
        .key("books.txt")
        // .body(stream)
        .content_type("text/plain")
        .send()
        .await
        .map_err(|err| {
            error!("Failed to upload books.txt to {}", bucket_name);
            "Failure"
        })?;


    // Return something that implements IntoResponse.
    // It will be serialized to the right response event automatically by the runtime
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body("Hello AWS Lambda HTTP request".into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}

#[cfg(test)]
mod tests {
    use lambda_http::aws_lambda_events::serde_json::json;
    use lambda_http::Service;
    use lambda_http::tower::ServiceExt;
    use lambda_http::Request;
    use super::*;
    use lambda_runtime::{Context, LambdaEvent};

    #[tokio::test]
    async fn test_func() -> () {
        let context = Context::default();
        let payload = json!({
            "field1": "value1",
            "field2": "value2",
            "field3": "value3"
        }).to_string();

        // let result = function_handler(event).await.unwrap();

        // let mut service = service_fn(function_handler);
        // let body = Body::from(payload.as_str());
        // let response = service
        //     .ready()
        //     .await?
        //     .call(Request::new(body))
        //     .await?;

        let request = Request::new(Body::from(payload.as_str()));

        let response = function_handler(request);

        let into_response = response.await;
        let ascii_body = into_response.unwrap().into_body().to_ascii_lowercase();
        let utf8_body = String::from_utf8(ascii_body).unwrap();

        assert_eq!("Hello AWS Lambda HTTP request".to_lowercase(), utf8_body);
    }

    // #[test]
    // fn test_json_to_parquet() {
    //     let payload = json!({
    //         "field1": "value1",
    //         "field2": "value2",
    //         "field3": "value3"
    //     }).to_string();
    //     let result = json_to_parquet(payload.as_str());
    // }
}
