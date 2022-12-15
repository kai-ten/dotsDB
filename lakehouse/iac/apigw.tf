resource "aws_apigatewayv2_api" "dotsdb_apigw_api" {
  name          = "dotsdb-http-api"
  protocol_type = "HTTP"
  target        = aws_lambda_function.dotsdb_ingestion_lambda.arn
}

resource "aws_apigatewayv2_integration" "dotsdb_ingest_integration" {
  description      = "Integration to dots ingestion lambda"
  api_id           = aws_apigatewayv2_api.dotsdb_apigw_api.id
  integration_type = "AWS_PROXY"

  connection_type    = "INTERNET"
  integration_method = "POST"
  integration_uri    = aws_lambda_function.dotsdb_ingestion_lambda.invoke_arn
}

resource "aws_apigatewayv2_route" "dotsdb_ingest_route" {
  api_id    = aws_apigatewayv2_api.dotsdb_apigw_api.id
  route_key = "POST /v1/books"

  target = "integrations/${aws_apigatewayv2_integration.dotsdb_ingest_integration.id}"
}

resource "aws_lambda_permission" "dotsdb_apigw_permissions" {
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.dotsdb_ingestion_lambda.arn
  principal     = "apigateway.amazonaws.com"

  source_arn = "${aws_apigatewayv2_api.dotsdb_apigw_api.execution_arn}/*/*"
}


# TODO: Add Authorizer for additional security
