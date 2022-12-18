module "dotsdb_apigw" {
    source = "./modules/api-gw"

    # Set in terraform.tfvars - 'false' if you do not want to create an API Gateway, 'true' if you do want to create an API Gateway
    # WARNING: Creating an API Gateway will create a publicly exposed endpoint. This is at your own risk.
    count = var.deploy_apigw == true ? 1 : 0

    dotsdb_ingest_lambda_arn = aws_lambda_function.dotsdb_ingestion_lambda.arn
    dotsdb_ingest_lambda_invoke_arn = aws_lambda_function.dotsdb_ingestion_lambda.invoke_arn
}