# To disable $default route, must configure custom domain - https://aws.amazon.com/premiumsupport/knowledge-center/api-gateway-disable-endpoint/
# Otherwise, all requests that don't match a route will be handled by the default route, which uses the dotsdb-ingestion-lambda

resource "aws_apigatewayv2_api" "dotsdb_apigw_api" {
  name          = "dotsdb-http-api"
  protocol_type = "HTTP"
  target        = var.dotsdb_ingest_lambda_arn
}

resource "aws_apigatewayv2_integration" "dotsdb_ingest_integration" {
  description      = "Integration to dots ingestion lambda"
  api_id           = aws_apigatewayv2_api.dotsdb_apigw_api.id
  integration_type = "AWS_PROXY"

  connection_type    = "INTERNET"
  integration_method = "POST"
  integration_uri    = var.dotsdb_ingest_lambda_invoke_arn
}

resource "aws_apigatewayv2_route" "dotsdb_ingest_route" {
  api_id    = aws_apigatewayv2_api.dotsdb_apigw_api.id
  route_key = "POST /books"

  target = "integrations/${aws_apigatewayv2_integration.dotsdb_ingest_integration.id}"
}

resource "aws_apigatewayv2_stage" "dotsdb_ingest_stage" {
  api_id = aws_apigatewayv2_api.dotsdb_apigw_api.id
  name   = "dotsdb_books_stage"
  auto_deploy = true

  access_log_settings {
    destination_arn = aws_cloudwatch_log_group.dotsdb_apigw_log_group.arn

    format = jsonencode({
      requestId               = "$context.requestId"
      sourceIp                = "$context.identity.sourceIp"
      requestTime             = "$context.requestTime"
      requestTimeEpoch        = "$context.requestTimeEpoch"
      protocol                = "$context.protocol"
      httpMethod              = "$context.httpMethod"
      resourcePath            = "$context.resourcePath"
      routeKey                = "$context.routeKey"
      status                  = "$context.status"
      responseLength          = "$context.responseLength"
      integrationErrorMessage = "$context.integrationErrorMessage"
    })
  }

  depends_on = [
    aws_apigatewayv2_route.dotsdb_ingest_route
  ]
}

resource "aws_apigatewayv2_deployment" "dots_db_deployment" {
  api_id      = aws_apigatewayv2_api.dotsdb_apigw_api.id
  description = "dotsDB Deployment"

  triggers = {
    redeployment = sha1(join(",", tolist([
      jsonencode(aws_apigatewayv2_integration.dotsdb_ingest_integration),
      jsonencode(aws_apigatewayv2_route.dotsdb_ingest_route),
    ])))
  }

  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_lambda_permission" "dotsdb_apigw_permissions" {
  action        = "lambda:InvokeFunction"
  function_name = var.dotsdb_ingest_lambda_arn
  principal     = "apigateway.amazonaws.com"

  source_arn = "${aws_apigatewayv2_api.dotsdb_apigw_api.execution_arn}/*/*"
}

resource "aws_cloudwatch_log_group" "dotsdb_apigw_log_group" {
  name = "/aws/api_gw/${aws_apigatewayv2_api.dotsdb_apigw_api.name}"

  retention_in_days = 3
}


# TODO: Add Authorizer for additional security
