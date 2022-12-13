package com.dotsdb

import org.apache.iceberg.PartitionSpec
import org.apache.iceberg.Schema
import org.apache.iceberg.aws.glue.GlueCatalog
import org.apache.iceberg.catalog.Catalog
import org.apache.iceberg.catalog.Namespace
import org.apache.iceberg.catalog.TableIdentifier
import org.apache.iceberg.types.Types

fun main() {}

public class Handler {
    fun handleRequest() {
        println("Starting...")
        val catalog: Catalog by lazy { DotsDBGlueCatalog.createIcebergCatalog() }
        val namespace = Namespace.of("dotsdb") // TODO: add tf env var import
        val tableId = TableIdentifier.of(namespace, "books") // TODO: add tf env var if expanding beyond 1 table, would require loop and config
        val schema = Schema(
            Types.NestedField.required(1, "marketplace", Types.StringType.get()),
            Types.NestedField.required(2, "customer_id", Types.StringType.get()),
            Types.NestedField.required(3, "review_id", Types.StringType.get()),
            Types.NestedField.required(4, "product_id", Types.StringType.get()),
            Types.NestedField.required(5, "product_parent", Types.StringType.get()),
            Types.NestedField.required(6, "product_title", Types.StringType.get()),
            Types.NestedField.required(7, "star_rating", Types.IntegerType.get()),
            Types.NestedField.required(8, "helpful_votes", Types.IntegerType.get()),
            Types.NestedField.required(9, "total_votes", Types.IntegerType.get()),
            Types.NestedField.required(10, "vine", Types.StringType.get()),
            Types.NestedField.required(11, "verified_purchase", Types.StringType.get()),
            Types.NestedField.required(12, "review_headline", Types.StringType.get()),
            Types.NestedField.required(13, "review_body", Types.StringType.get()),
            Types.NestedField.required(14, "review_date", Types.DateType.get()),
            Types.NestedField.required(15, "year", Types.IntegerType.get())
        )
        val spec = PartitionSpec.builderFor(schema)
            .day("review_date")
            .build()

        // https://iceberg.apache.org/docs/latest/configuration/
        // Keep an eye on lz4 support for Athena (no avro support at time of writing - https://docs.aws.amazon.com/athena/latest/ug/compression-formats.html)
        val properties = mapOf(
            "write.format.default" to "parquet",
            "write.avro.compression-codec" to "snappy",
            "write.parquet.compression-codec" to "snappy",
            "format-version" to "2" // https://iceberg.apache.org/spec/#format-versioning
        )

        if (!catalog.tableExists(tableId)) {
            catalog.createTable(tableId, schema, spec, properties)
        } else if (!catalog.tableExists(tableId)) {
            // handle update
        } else {
            // handle delete
        }
    }
}


class DotsDBGlueCatalog {
    companion object {
        fun createIcebergCatalog(): GlueCatalog {
            return GlueCatalog().apply { initialize("glue_catalog", icebergProperties) }
        }

        // https://iceberg.apache.org/docs/latest/aws/
        private val icebergProperties = mapOf(
            "name" to "dotsdb",
            "type" to "iceberg",
            "warehouse" to "s3://dotsdb-lakehouse-data/books", // TODO: add tf env var import
            "catalog-impl" to "org.apache.iceberg.aws.glue.GlueCatalog",
            "io-impl" to "org.apache.iceberg.aws.s3.S3FileIO",
            "s3.dualstack-enabled" to "true",
            "write.object-storage.enabled" to "true",
            "write.data.path" to "s3://dotsdb-lakehouse-data/books", // TODO: add tf env var import
            "s3.write.tags.owner" to "dotsDB"
        )
    }
}
