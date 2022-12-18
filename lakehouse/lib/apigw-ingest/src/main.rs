mod book_reviews;

// use arrow2::io::json::read;
use async_once::AsyncOnce;
use aws_config::SdkConfig;
use aws_sdk_s3::Client;
use aws_sdk_s3::types::ByteStream;
use lambda_http::aws_lambda_events::event;
use lambda_http::Body;
use lambda_http::ext::PayloadError;
use lambda_http::http::{HeaderMap, StatusCode};
use lambda_http::request::RequestContext;
use lambda_runtime::{Error, LambdaEvent, run, service_fn};
use log::error;
use serde_json::json;
use crate::book_reviews::BookReview;
use serde::{Deserialize, Serialize};
use crate::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};
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

pub async fn function_handler(event: LambdaEvent<ApiGatewayProxyRequest>) -> Result<ApiGatewayProxyResponse, Error> {
    let body = event.payload.body.unwrap_or("".to_string());
    println!("event: {:?}", body);


    // TODO: error handle json body
    let resp_body = Body::Text(body);

    let resp = ApiGatewayProxyResponse {
        status_code: 200,
        body: Option::from(resp_body),
        is_base64_encoded: Option::from(false),
        headers: HeaderMap::new(),
        multi_value_headers: HeaderMap::new()
    };

    Ok(resp)
}


//     // TODO: let bucket_name = std::env::var("BUCKET_NAME")
//     //     .expect("Set a bucket name in the environment variables.");

//     let bucket_name = "";
//     let s3_client: &Client = S3_CLIENT.get().await;

//     // TODO: should have 3 different s3 writes - data, manifest.avro, schema.json
//     let _ = s3_client.put_object()
//         .bucket(bucket_name)
//         .key("books.txt")
//         // .body(stream)
//         .content_type("text/plain")
//         .send()
//         .await
//         .map_err(|err| {
//             err.to_string()
//         })?;


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
    use lambda_http::http::header::{CONTENT_TYPE, HOST};
    use lambda_http::Service;
    use lambda_http::tower::ServiceExt;
    use lambda_http::Request;
    use super::*;
    use lambda_runtime::{Context, LambdaEvent};

    #[tokio::test]
    async fn test_func() -> () {
        let context = Context::default();

        let mut headers = HeaderMap::new();
        headers.insert(HOST, "test.com".parse().unwrap());
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

        // Example data: https://github.com/awslabs/aws-lambda-rust-runtime/blob/f8706e332ee1732284c9b51c816df99d264bd39e/lambda-http/tests/data/apigw_proxy_request.json
        let mut apigw_v2 = ApiGatewayProxyRequest::default();
        apigw_v2.headers = headers;
        apigw_v2.body = Option::from("Hello AWS Lambda HTTP request".to_string());

        let request = LambdaEvent::new(apigw_v2, context);
        let response = function_handler(request);


        let into_response = response.await;
        let body = into_response.unwrap().body.unwrap().to_vec();
        let utf8_body = String::from_utf8(body).unwrap();

        assert_eq!("Hello AWS Lambda HTTP request", utf8_body);
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
