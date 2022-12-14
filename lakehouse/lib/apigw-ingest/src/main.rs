use std::env;
use arrow2::array::Array;
use arrow2::chunk::Chunk;
use arrow2::datatypes::{Schema, Field};
use async_once::AsyncOnce;
use arrow2::io::json::read;
use arrow2::datatypes::DataType::{Int16, Int64, Int8, LargeUtf8, Utf8, Struct};
use arrow2::io::parquet::write::{CompressionOptions, Encoding, FileSink, to_parquet_schema, transverse, Version, WriteOptions};
use aws_config::{SdkConfig};
use aws_sdk_s3::types::ByteStream;
use futures::SinkExt;
use lambda_http::aws_lambda_events::event;
use lambda_http::Body;
use lambda_http::http::{HeaderMap};
use lambda_runtime::{Error, LambdaEvent, run, service_fn};
use tokio::fs::File;
use tokio_util::compat::TokioAsyncReadCompatExt;
use uuid::Uuid;
// use log::error;
// use serde_json::Value;
// use tokio::fs::File;
// use tokio_util::compat::TokioAsyncReadCompatExt;
use crate::event::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse};


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
// 3. Write the manifest.avro to the s3 metadata folder
// 4. Write the metadata.json to the s3 metadata folder

lazy_static! (
    static ref AWS_CONFIG: AsyncOnce<SdkConfig> = AsyncOnce::new(async { aws_config::load_from_env().await });
    static ref S3_CLIENT: AsyncOnce<aws_sdk_s3::Client> = AsyncOnce::new(async { aws_sdk_s3::Client::new(AWS_CONFIG.get().await) });
);


async fn write_chunk(path: &str, schema: Schema, chunk: Chunk<Box<dyn Array>>) -> Result<(), anyhow::Error> {
    let options = WriteOptions {
        write_statistics: false,
        compression: CompressionOptions::Uncompressed,
        version: Version::V2
    };

    let mut stream = futures::stream::iter(vec![Ok(chunk)].into_iter());

    let encodings: Vec<Vec<Encoding>> = schema
        .fields
        .iter()
        .map(|f| transverse(&f.data_type, |_| Encoding::Plain))
        .collect();

    let file = File::create(path).await?.compat();

    let mut sink = FileSink::try_new(file, schema, encodings, options)?;
    sink.send_all(&mut stream).await?;
    sink.close().await?;

    Ok(())
}

pub async fn function_handler(event: LambdaEvent<ApiGatewayProxyRequest>) -> Result<ApiGatewayProxyResponse, Error> {

    let write_file_path = "/tmp/test.parquet";

    // In the real world, Field and Schema will be generated by some sort of config as code, depending on the event source
    let book_review_field = Field::new(
        "book_review",
        Struct(vec![
            Field::new("marketplace", Utf8, true),
            Field::new("customer_id", Utf8, true),
            Field::new("review_id", Utf8, true),
            Field::new("product_id", Utf8, true),
            Field::new("product_parent", Utf8, true),
            Field::new("product_title", Utf8, true),
            Field::new("star_rating", Int8, true),
            Field::new("helpful_votes", Int64, true),
            Field::new("total_votes", Int64, true),
            Field::new("vine", Utf8, true),
            Field::new("verified_purchase", Utf8, true),
            Field::new("review_headline", Utf8, true),
            Field::new("review_body", LargeUtf8, true),
            Field::new("review_date", Utf8, true),
            Field::new("year", Int16, true),
        ]),
        true
    );

    let book_review_schema = Schema::from(vec![
        book_review_field
    ]);

    let body = event.payload.body.unwrap_or_else(|| "".to_string());
    let json_bytes = read::json_deserializer::parse(body.as_bytes()).unwrap();
    let data_type = read::infer(&json_bytes)?;
    let data = read::deserialize(&json_bytes, data_type).unwrap();
    let chunk = Chunk::new(vec![data]);

    write_chunk(write_file_path, book_review_schema, chunk).await.unwrap();

    let body = ByteStream::from_path(write_file_path).await.expect("File not found.");
    let s3 = S3_CLIENT.get().await;
    s3
        .put_object()
        .bucket(env::var("DOTSDB_DATA_BUCKET").unwrap().to_string())
        .key(Uuid::new_v4().to_string() + ".parquet") // "books/data/file.snappy.parquet" is ultimate goal
        .body(body)
        .send()
        .await?;


    let resp = ApiGatewayProxyResponse {
        status_code: 200,
        body: Option::from(Body::Text("Success".to_string())),
        is_base64_encoded: Option::from(false),
        headers: HeaderMap::new(),
        multi_value_headers: HeaderMap::new()
    };

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
    use lambda_http::http::header::{CONTENT_TYPE, HOST};
    use super::*;
    use lambda_runtime::{Context, LambdaEvent};

    #[tokio::test]
    async fn test_func() {
        env::set_var("DOTSDB_DATA_BUCKET", "dotsdb-lakehouse-data");
        let context = Context::default();

        let mut headers = HeaderMap::new();
        headers.insert(HOST, "test.com".parse().unwrap());
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

        // Example data: https://github.com/awslabs/aws-lambda-rust-runtime/blob/f8706e332ee1732284c9b51c816df99d264bd39e/lambda-http/tests/data/apigw_proxy_request.json
        let mut apigw_v2 = ApiGatewayProxyRequest::default();
        apigw_v2.headers = headers;
        apigw_v2.body = Option::from("[\n    {\n        \"marketplace\": \"US\",\n        \"customer_id\": \"10822695\",\n        \"review_id\": \"R2RRIALQ1UBYO8\",\n        \"product_id\": \"0385418493\",\n        \"product_parent\": \"610658517\",\n        \"product_title\": \"How the Irish Saved Civilization: The Untold Story of Ireland's Heroic Role From the Fall of Rome to the Rise of Medieval Europe (The Hinges of History)\",\n        \"star_rating\": 1,\n        \"helpful_votes\": 153,\n        \"total_votes\": 169,\n        \"vine\": \"N\",\n        \"verified_purchase\": \"N\",\n        \"review_headline\": \"Total Rubbish.\",\n        \"review_body\": \"The last reviwer is a bit daft. In some 70 years of reading History I have never read such lies, distortions, and incoherent gibberish. The author is CLEARLY appealing to ethnic sentiment over \\\"EVIDENCE AND FACTS.\\\" I suggest readers read the dozen or so \\\"Most Helpful Reviews.\\\" Those reviewers were very in depth and know their SUBJECT.\",\n        \"review_date\": \"2006-06-11\",\n        \"year\": 2006\n    },\n    {\n        \"marketplace\": \"US\",\n        \"customer_id\": \"10822695\",\n        \"review_id\": \"R2RRIALQ1UBYO8\",\n        \"product_id\": \"0385418493\",\n        \"product_parent\": \"610658517\",\n        \"product_title\": \"How the Irish Saved Civilization: The Untold Story of Ireland's Heroic Role From the Fall of Rome to the Rise of Medieval Europe (The Hinges of History)\",\n        \"star_rating\": 1,\n        \"helpful_votes\": 153,\n        \"total_votes\": 169,\n        \"vine\": \"N\",\n        \"verified_purchase\": \"N\",\n        \"review_headline\": \"Total Rubbish.\",\n        \"review_body\": \"The last reviwer is a bit daft. In some 70 years of reading History I have never read such lies, distortions, and incoherent gibberish. The author is CLEARLY appealing to ethnic sentiment over \\\"EVIDENCE AND FACTS.\\\" I suggest readers read the dozen or so \\\"Most Helpful Reviews.\\\" Those reviewers were very in depth and know their SUBJECT.\",\n        \"review_date\": \"2006-06-11\",\n        \"year\": 2006\n    },\n    {\n        \"marketplace\": \"US\",\n        \"customer_id\": \"10822695\",\n        \"review_id\": \"R2RRIALQ1UBYO8\",\n        \"product_id\": \"0385418493\",\n        \"product_parent\": \"610658517\",\n        \"product_title\": \"How the Irish Saved Civilization: The Untold Story of Ireland's Heroic Role From the Fall of Rome to the Rise of Medieval Europe (The Hinges of History)\",\n        \"star_rating\": 1,\n        \"helpful_votes\": 153,\n        \"total_votes\": 169,\n        \"vine\": \"N\",\n        \"verified_purchase\": \"N\",\n        \"review_headline\": \"Total Rubbish.\",\n        \"review_body\": \"The last reviwer is a bit daft. In some 70 years of reading History I have never read such lies, distortions, and incoherent gibberish. The author is CLEARLY appealing to ethnic sentiment over \\\"EVIDENCE AND FACTS.\\\" I suggest readers read the dozen or so \\\"Most Helpful Reviews.\\\" Those reviewers were very in depth and know their SUBJECT.\",\n        \"review_date\": \"2006-06-11\",\n        \"year\": 2006\n    },\n    {\n        \"marketplace\": \"US\",\n        \"customer_id\": \"10822695\",\n        \"review_id\": \"R2RRIALQ1UBYO8\",\n        \"product_id\": \"0385418493\",\n        \"product_parent\": \"610658517\",\n        \"product_title\": \"How the Irish Saved Civilization: The Untold Story of Ireland's Heroic Role From the Fall of Rome to the Rise of Medieval Europe (The Hinges of History)\",\n        \"star_rating\": 1,\n        \"helpful_votes\": 153,\n        \"total_votes\": 169,\n        \"vine\": \"N\",\n        \"verified_purchase\": \"N\",\n        \"review_headline\": \"Total Rubbish.\",\n        \"review_body\": \"The last reviwer is a bit daft. In some 70 years of reading History I have never read such lies, distortions, and incoherent gibberish. The author is CLEARLY appealing to ethnic sentiment over \\\"EVIDENCE AND FACTS.\\\" I suggest readers read the dozen or so \\\"Most Helpful Reviews.\\\" Those reviewers were very in depth and know their SUBJECT.\",\n        \"review_date\": \"2006-06-11\",\n        \"year\": 2006\n    },\n    {\n        \"marketplace\": \"US\",\n        \"customer_id\": \"10822695\",\n        \"review_id\": \"R2RRIALQ1UBYO8\",\n        \"product_id\": \"0385418493\",\n        \"product_parent\": \"610658517\",\n        \"product_title\": \"How the Irish Saved Civilization: The Untold Story of Ireland's Heroic Role From the Fall of Rome to the Rise of Medieval Europe (The Hinges of History)\",\n        \"star_rating\": 1,\n        \"helpful_votes\": 153,\n        \"total_votes\": 169,\n        \"vine\": \"N\",\n        \"verified_purchase\": \"N\",\n        \"review_headline\": \"Total Rubbish.\",\n        \"review_body\": \"The last reviwer is a bit daft. In some 70 years of reading History I have never read such lies, distortions, and incoherent gibberish. The author is CLEARLY appealing to ethnic sentiment over \\\"EVIDENCE AND FACTS.\\\" I suggest readers read the dozen or so \\\"Most Helpful Reviews.\\\" Those reviewers were very in depth and know their SUBJECT.\",\n        \"review_date\": \"2006-06-11\",\n        \"year\": 2006\n    },\n    {\n        \"marketplace\": \"US\",\n        \"customer_id\": \"10822695\",\n        \"review_id\": \"R2RRIALQ1UBYO8\",\n        \"product_id\": \"0385418493\",\n        \"product_parent\": \"610658517\",\n        \"product_title\": \"How the Irish Saved Civilization: The Untold Story of Ireland's Heroic Role From the Fall of Rome to the Rise of Medieval Europe (The Hinges of History)\",\n        \"star_rating\": 1,\n        \"helpful_votes\": 153,\n        \"total_votes\": 169,\n        \"vine\": \"N\",\n        \"verified_purchase\": \"N\",\n        \"review_headline\": \"Total Rubbish.\",\n        \"review_body\": \"The last reviwer is a bit daft. In some 70 years of reading History I have never read such lies, distortions, and incoherent gibberish. The author is CLEARLY appealing to ethnic sentiment over \\\"EVIDENCE AND FACTS.\\\" I suggest readers read the dozen or so \\\"Most Helpful Reviews.\\\" Those reviewers were very in depth and know their SUBJECT.\",\n        \"review_date\": \"2006-06-11\",\n        \"year\": 2006\n    },\n    {\n        \"marketplace\": \"US\",\n        \"customer_id\": \"10822695\",\n        \"review_id\": \"R2RRIALQ1UBYO8\",\n        \"product_id\": \"0385418493\",\n        \"product_parent\": \"610658517\",\n        \"product_title\": \"How the Irish Saved Civilization: The Untold Story of Ireland's Heroic Role From the Fall of Rome to the Rise of Medieval Europe (The Hinges of History)\",\n        \"star_rating\": 1,\n        \"helpful_votes\": 153,\n        \"total_votes\": 169,\n        \"vine\": \"N\",\n        \"verified_purchase\": \"N\",\n        \"review_headline\": \"Total Rubbish.\",\n        \"review_body\": \"The last reviwer is a bit daft. In some 70 years of reading History I have never read such lies, distortions, and incoherent gibberish. The author is CLEARLY appealing to ethnic sentiment over \\\"EVIDENCE AND FACTS.\\\" I suggest readers read the dozen or so \\\"Most Helpful Reviews.\\\" Those reviewers were very in depth and know their SUBJECT.\",\n        \"review_date\": \"2006-06-11\",\n        \"year\": 2006\n    },\n    {\n        \"marketplace\": \"US\",\n        \"customer_id\": \"10822695\",\n        \"review_id\": \"R2RRIALQ1UBYO8\",\n        \"product_id\": \"0385418493\",\n        \"product_parent\": \"610658517\",\n        \"product_title\": \"How the Irish Saved Civilization: The Untold Story of Ireland's Heroic Role From the Fall of Rome to the Rise of Medieval Europe (The Hinges of History)\",\n        \"star_rating\": 1,\n        \"helpful_votes\": 153,\n        \"total_votes\": 169,\n        \"vine\": \"N\",\n        \"verified_purchase\": \"N\",\n        \"review_headline\": \"Total Rubbish.\",\n        \"review_body\": \"The last reviwer is a bit daft. In some 70 years of reading History I have never read such lies, distortions, and incoherent gibberish. The author is CLEARLY appealing to ethnic sentiment over \\\"EVIDENCE AND FACTS.\\\" I suggest readers read the dozen or so \\\"Most Helpful Reviews.\\\" Those reviewers were very in depth and know their SUBJECT.\",\n        \"review_date\": \"2006-06-11\",\n        \"year\": 2006\n    },\n    {\n        \"marketplace\": \"US\",\n        \"customer_id\": \"10822695\",\n        \"review_id\": \"R2RRIALQ1UBYO8\",\n        \"product_id\": \"0385418493\",\n        \"product_parent\": \"610658517\",\n        \"product_title\": \"How the Irish Saved Civilization: The Untold Story of Ireland's Heroic Role From the Fall of Rome to the Rise of Medieval Europe (The Hinges of History)\",\n        \"star_rating\": 1,\n        \"helpful_votes\": 153,\n        \"total_votes\": 169,\n        \"vine\": \"N\",\n        \"verified_purchase\": \"N\",\n        \"review_headline\": \"Total Rubbish.\",\n        \"review_body\": \"The last reviwer is a bit daft. In some 70 years of reading History I have never read such lies, distortions, and incoherent gibberish. The author is CLEARLY appealing to ethnic sentiment over \\\"EVIDENCE AND FACTS.\\\" I suggest readers read the dozen or so \\\"Most Helpful Reviews.\\\" Those reviewers were very in depth and know their SUBJECT.\",\n        \"review_date\": \"2006-06-11\",\n        \"year\": 2006\n    },\n    {\n        \"marketplace\": \"US\",\n        \"customer_id\": \"10822695\",\n        \"review_id\": \"R2RRIALQ1UBYO8\",\n        \"product_id\": \"0385418493\",\n        \"product_parent\": \"610658517\",\n        \"product_title\": \"How the Irish Saved Civilization: The Untold Story of Ireland's Heroic Role From the Fall of Rome to the Rise of Medieval Europe (The Hinges of History)\",\n        \"star_rating\": 1,\n        \"helpful_votes\": 153,\n        \"total_votes\": 169,\n        \"vine\": \"N\",\n        \"verified_purchase\": \"N\",\n        \"review_headline\": \"Total Rubbish.\",\n        \"review_body\": \"The last reviwer is a bit daft. In some 70 years of reading History I have never read such lies, distortions, and incoherent gibberish. The author is CLEARLY appealing to ethnic sentiment over \\\"EVIDENCE AND FACTS.\\\" I suggest readers read the dozen or so \\\"Most Helpful Reviews.\\\" Those reviewers were very in depth and know their SUBJECT.\",\n        \"review_date\": \"2006-06-11\",\n        \"year\": 2006\n    },\n    {\n        \"marketplace\": \"US\",\n        \"customer_id\": \"10822695\",\n        \"review_id\": \"R2RRIALQ1UBYO8\",\n        \"product_id\": \"0385418493\",\n        \"product_parent\": \"610658517\",\n        \"product_title\": \"How the Irish Saved Civilization: The Untold Story of Ireland's Heroic Role From the Fall of Rome to the Rise of Medieval Europe (The Hinges of History)\",\n        \"star_rating\": 1,\n        \"helpful_votes\": 153,\n        \"total_votes\": 169,\n        \"vine\": \"N\",\n        \"verified_purchase\": \"N\",\n        \"review_headline\": \"Total Rubbish.\",\n        \"review_body\": \"The last reviwer is a bit daft. In some 70 years of reading History I have never read such lies, distortions, and incoherent gibberish. The author is CLEARLY appealing to ethnic sentiment over \\\"EVIDENCE AND FACTS.\\\" I suggest readers read the dozen or so \\\"Most Helpful Reviews.\\\" Those reviewers were very in depth and know their SUBJECT.\",\n        \"review_date\": \"2006-06-11\",\n        \"year\": 2006\n    },\n    {\n        \"marketplace\": \"US\",\n        \"customer_id\": \"10822695\",\n        \"review_id\": \"R2RRIALQ1UBYO8\",\n        \"product_id\": \"0385418493\",\n        \"product_parent\": \"610658517\",\n        \"product_title\": \"How the Irish Saved Civilization: The Untold Story of Ireland's Heroic Role From the Fall of Rome to the Rise of Medieval Europe (The Hinges of History)\",\n        \"star_rating\": 1,\n        \"helpful_votes\": 153,\n        \"total_votes\": 169,\n        \"vine\": \"N\",\n        \"verified_purchase\": \"N\",\n        \"review_headline\": \"Total Rubbish.\",\n        \"review_body\": \"The last reviwer is a bit daft. In some 70 years of reading History I have never read such lies, distortions, and incoherent gibberish. The author is CLEARLY appealing to ethnic sentiment over \\\"EVIDENCE AND FACTS.\\\" I suggest readers read the dozen or so \\\"Most Helpful Reviews.\\\" Those reviewers were very in depth and know their SUBJECT.\",\n        \"review_date\": \"2006-06-11\",\n        \"year\": 2006\n    },\n    {\n        \"marketplace\": \"US\",\n        \"customer_id\": \"10822695\",\n        \"review_id\": \"R2RRIALQ1UBYO8\",\n        \"product_id\": \"0385418493\",\n        \"product_parent\": \"610658517\",\n        \"product_title\": \"How the Irish Saved Civilization: The Untold Story of Ireland's Heroic Role From the Fall of Rome to the Rise of Medieval Europe (The Hinges of History)\",\n        \"star_rating\": 1,\n        \"helpful_votes\": 153,\n        \"total_votes\": 169,\n        \"vine\": \"N\",\n        \"verified_purchase\": \"N\",\n        \"review_headline\": \"Total Rubbish.\",\n        \"review_body\": \"The last reviwer is a bit daft. In some 70 years of reading History I have never read such lies, distortions, and incoherent gibberish. The author is CLEARLY appealing to ethnic sentiment over \\\"EVIDENCE AND FACTS.\\\" I suggest readers read the dozen or so \\\"Most Helpful Reviews.\\\" Those reviewers were very in depth and know their SUBJECT.\",\n        \"review_date\": \"2006-06-11\",\n        \"year\": 2006\n    },\n    {\n        \"marketplace\": \"US\",\n        \"customer_id\": \"10822695\",\n        \"review_id\": \"R2RRIALQ1UBYO8\",\n        \"product_id\": \"0385418493\",\n        \"product_parent\": \"610658517\",\n        \"product_title\": \"How the Irish Saved Civilization: The Untold Story of Ireland's Heroic Role From the Fall of Rome to the Rise of Medieval Europe (The Hinges of History)\",\n        \"star_rating\": 1,\n        \"helpful_votes\": 153,\n        \"total_votes\": 169,\n        \"vine\": \"N\",\n        \"verified_purchase\": \"N\",\n        \"review_headline\": \"Total Rubbish.\",\n        \"review_body\": \"The last reviwer is a bit daft. In some 70 years of reading History I have never read such lies, distortions, and incoherent gibberish. The author is CLEARLY appealing to ethnic sentiment over \\\"EVIDENCE AND FACTS.\\\" I suggest readers read the dozen or so \\\"Most Helpful Reviews.\\\" Those reviewers were very in depth and know their SUBJECT.\",\n        \"review_date\": \"2006-06-11\",\n        \"year\": 2006\n    },\n    {\n        \"marketplace\": \"US\",\n        \"customer_id\": \"10822695\",\n        \"review_id\": \"R2RRIALQ1UBYO8\",\n        \"product_id\": \"0385418493\",\n        \"product_parent\": \"610658517\",\n        \"product_title\": \"How the Irish Saved Civilization: The Untold Story of Ireland's Heroic Role From the Fall of Rome to the Rise of Medieval Europe (The Hinges of History)\",\n        \"star_rating\": 1,\n        \"helpful_votes\": 153,\n        \"total_votes\": 169,\n        \"vine\": \"N\",\n        \"verified_purchase\": \"N\",\n        \"review_headline\": \"Total Rubbish.\",\n        \"review_body\": \"The last reviwer is a bit daft. In some 70 years of reading History I have never read such lies, distortions, and incoherent gibberish. The author is CLEARLY appealing to ethnic sentiment over \\\"EVIDENCE AND FACTS.\\\" I suggest readers read the dozen or so \\\"Most Helpful Reviews.\\\" Those reviewers were very in depth and know their SUBJECT.\",\n        \"review_date\": \"2006-06-11\",\n        \"year\": 2006\n    }\n]".to_string());

        let request = LambdaEvent::new(apigw_v2, context);
        let response = function_handler(request);


        let into_response = response.await;
        let body = into_response.unwrap().body.unwrap().to_vec();
        let utf8_body = String::from_utf8(body).unwrap();

        assert_eq!("Success", utf8_body);
    }

}
