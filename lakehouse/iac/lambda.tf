resource "aws_iam_role" "dotsdb_iceberg_lambda_iam" {
  name = "iam_for_lambda"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Principal = {
          Service = [
            "lambda.amazonaws.com",
          ]
        }
        Action = [
          "sts:AssumeRole"
        ]
        Effect = "Allow"
      },
    ]
  })
}

resource "aws_iam_policy" "dotsdb_iceberg_lambda_iam_policy" {
  name        = "lambda_logging"
  path        = "/"
  description = "IAM policy for logging from a lambda"

  policy = jsonencode({
    Version = "2012-10-17"
    # TODO: Make policy more restrictive
    Statement = [
      {
        Action = [
          "s3:*"
        ]
        Effect   = "Allow"
        Resource = "*"
      },
      {
        Effect = "Allow",
        Action = [
          "glue:*"
        ],
        Resource = [
          "arn:aws:glue:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:*"
        ]
      },
      {
        Effect = "Allow",
        Action = [
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ],
        Resource = [
          "arn:aws:logs:*:*:*",
        ]
      },
      {
        Action = [
          "kms:Decrypt"
        ]
        Effect   = "Allow"
        Resource = "${data.aws_kms_key.aws_s3_kms.arn}"
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "lambda_logs" {
  role       = aws_iam_role.dotsdb_iceberg_lambda_iam.name
  policy_arn = aws_iam_policy.dotsdb_iceberg_lambda_iam_policy.arn
}

locals {
  jar_file = "./../iceberg/build/libs/iceberg-0.1-uber.jar"
}

resource "aws_s3_object" "file_upload" {
  bucket = "${aws_s3_bucket.dotsdb_iceberg_lambda.id}"
  key    = "lambda-functions/iceberg.zip"
  source = local.jar_file
  source_hash = filemd5("${local.jar_file}")
}

resource "aws_lambda_function" "dotsdb_iceberg_lambda" {
  function_name    = "dotsDB-Iceberg-Table-Manager"
  role             = aws_iam_role.dotsdb_iceberg_lambda_iam.arn
  handler          = "com.dotsdb.Handler::handleRequest"
  s3_bucket        = aws_s3_object.file_upload.bucket
  s3_key           = aws_s3_object.file_upload.key
  source_code_hash = filebase64sha256("${local.jar_file}")
  memory_size      = 1024
  timeout = 30
  runtime = "java11"

  environment {
    variables = {
      # TODO: Add all env vars here for lambda
      DOTSDB_DATA_BUCKET_NAME = var.data_s3_bucket
      DOTSDB_NAMESPACE        = "dotsdb"
    }
  }
}