data "aws_kms_key" "aws_s3_kms" {
  key_id = "alias/aws/s3"
}

data "aws_iam_policy" "managed_glue_role_policy" {
  arn = "arn:aws:iam::aws:policy/service-role/AWSGlueServiceRole"
}

resource "aws_iam_policy" "glue_crawler_target_policy" {
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = [
          "s3:GetObject",
          "s3:PutObject"
        ]
        Effect   = "Allow"
        Resource = "${module.dotsdb_data_bucket.data.arn}/*"
      },
      {
        Effect = "Allow",
        Action = [
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:DescribeLogStreams",
          "logs:PutLogEvents"
        ],
        Resource = [
          "arn:aws:logs:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:log-group:/aws-glue/crawlers/*",
        ]
      },
      {
        Action = [
          "kms:Decrypt"
        ]
        Effect   = "Allow"
        Resource = "${data.aws_kms_key.aws_s3_kms.arn}"
      },
    ]
  })
}

resource "aws_iam_role" "crawler_role" {
  name = "dotsdb-crawler-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Principal = {
          Service = [
            "glue.amazonaws.com",
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

resource "aws_iam_role_policy_attachment" "awsglueservicerole_attachment" {
  role       = aws_iam_role.crawler_role.name
  policy_arn = data.aws_iam_policy.managed_glue_role_policy.arn
}

resource "aws_iam_role_policy_attachment" "s3_attachment" {
  role       = aws_iam_role.crawler_role.name
  policy_arn = aws_iam_policy.glue_crawler_target_policy.arn
}

resource "aws_glue_registry" "dotsdb_lake_glue_registry" {
  registry_name = "dotsDB-Registry"
}

resource "aws_glue_catalog_database" "dotsdb_lake_glue_catalog_database" {
  name         = "dotsdb"
  description  = "dotsDB Iceberg Database"
  location_uri = "s3://${module.dotsdb_data_bucket.data.bucket}"
  catalog_id   = data.aws_caller_identity.current.account_id
}
