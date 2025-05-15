mod schema;

use crate::schema::{infer_scheam_from_data_file, read_schema_from_schema_file_in_json};
use arrow::csv::ReaderBuilder;
use clap::{Parser, ValueHint};
use parquet::basic::{BrotliLevel, GzipLevel, ZstdLevel};
use parquet::{
    arrow::ArrowWriter,
    basic::{Compression, Encoding},
    errors::ParquetError,
    file::properties::{EnabledStatistics, WriterProperties},
};
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(clap::ValueEnum, Clone)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum ParquetCompression {
    UNCOMPRESSED,
    SNAPPY,
    GZIP,
    LZO,
    BROTLI,
    LZ4,
    ZSTD,
}

#[derive(clap::ValueEnum, Clone)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum ParquetEncoding {
    PLAIN,
    RLE,
    DELTA_BINARY_PACKED,
    DELTA_LENGTH_BYTE_ARRAY,
    DELTA_BYTE_ARRAY,
    RLE_DICTIONARY,
}

#[derive(clap::ValueEnum, Clone)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum ParquetEnabledStatistics {
    None,
    Chunk,
    Page,
}

#[derive(Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = "Frank Lan <franklan118@gmail.com>")]
struct Options {
    /// Input CSV file.
    #[clap(name = "CSV", value_parser, value_hint = ValueHint::AnyPath)]
    input: PathBuf,

    /// Output file.
    #[clap(name = "PARQUET", value_parser, value_hint = ValueHint::AnyPath)]
    output: PathBuf,

    /// File with Arrow schema in JSON format.
    #[clap(short = 's', long, value_parser, value_hint = ValueHint::AnyPath)]
    schema_file: Option<PathBuf>,

    /// The number of records to infer the schema from. All rows if not present. Setting max-read-records to zero will stop schema inference and all columns will be string typed.
    #[clap(long)]
    max_read_records: Option<usize>,

    /// Set whether the CSV file has headers
    #[clap(long)]
    header: Option<bool>,

    /// Set the CSV file's column delimiter as a byte character.
    #[clap(short, long, default_value = ",")]
    delimiter: char,

    /// Set the compression.
    #[clap(short, long, value_enum)]
    compression: Option<ParquetCompression>,

    /// Sets encoding for any column.
    #[clap(short, long, value_enum)]
    encoding: Option<ParquetEncoding>,

    /// Sets data page size limit.
    #[clap(long)]
    data_pagesize_limit: Option<usize>,

    /// Sets dictionary page size limit.
    #[clap(long)]
    dictionary_pagesize_limit: Option<usize>,

    /// Sets write batch size.
    #[clap(long)]
    write_batch_size: Option<usize>,

    /// Sets max size for a row group.
    #[clap(long)]
    max_row_group_size: Option<usize>,

    /// Sets "created by" property.
    #[clap(long)]
    created_by: Option<String>,

    /// Sets flag to enable/disable dictionary encoding for any column.
    #[clap(long)]
    dictionary: bool,

    /// Sets flag to enable/disable statistics for any column.
    #[clap(long, value_enum)]
    statistics: Option<ParquetEnabledStatistics>,

    /// Print the schema to stderr.
    #[clap(short, long)]
    print_schema: bool,

    /// Only print the schema
    #[clap(short = 'n', long)]
    dry: bool,
}

fn main() -> Result<(), ParquetError> {
    let opts: Options = Options::parse();
    csv_to_parquet(opts)
}

fn csv_to_parquet(options: Options) -> Result<(), ParquetError> {
    let input = File::open(options.input.clone())?;

    let schema = match options.schema_file {
        Some(schema_def_file_path) => {
            let schema_file_path = schema_def_file_path
                .to_str()
                .expect("Failed to convert path to string")
                .to_string();
            read_schema_from_schema_file_in_json(schema_file_path)
        }
        _ => {
            let input_file_path = options
                .input
                .clone()
                .to_str()
                .expect("Failed to convert path to string")
                .to_string();
            infer_scheam_from_data_file(
                input_file_path,
                options.delimiter as u8,
                options.max_read_records,
                options.header.unwrap_or(true),
            )
        }
    };

    if options.print_schema || options.dry {
        let json = serde_json::to_string_pretty(&schema).unwrap();
        eprintln!("Schema:");
        println!("{}", json);
        if options.dry {
            return Ok(());
        }
    }

    let schema_ref = Arc::new(schema);
    let builder = ReaderBuilder::new(schema_ref)
        .with_header(options.header.unwrap_or(true))
        .with_delimiter(options.delimiter as u8);

    let reader = builder.build(input)?;

    let output = File::create(options.output)?;

    let mut props = WriterProperties::builder().set_dictionary_enabled(options.dictionary);

    if let Some(statistics) = options.statistics {
        let statistics = match statistics {
            ParquetEnabledStatistics::Chunk => EnabledStatistics::Chunk,
            ParquetEnabledStatistics::Page => EnabledStatistics::Page,
            ParquetEnabledStatistics::None => EnabledStatistics::None,
        };

        props = props.set_statistics_enabled(statistics);
    }

    if let Some(compression) = options.compression {
        let compression = match compression {
            ParquetCompression::UNCOMPRESSED => Compression::UNCOMPRESSED,
            ParquetCompression::SNAPPY => Compression::SNAPPY,
            ParquetCompression::GZIP => {
                let gzip_level: GzipLevel =
                    GzipLevel::try_new(9).expect("Failed to create GzipLevel");
                Compression::GZIP(gzip_level)
            }
            ParquetCompression::LZO => Compression::LZO,
            ParquetCompression::BROTLI => {
                let brotli_level: BrotliLevel =
                    BrotliLevel::try_new(11).expect("Failed to create BrotliLevel");
                Compression::BROTLI(brotli_level)
            }
            ParquetCompression::LZ4 => Compression::LZ4,
            ParquetCompression::ZSTD => {
                let zstd_level = ZstdLevel::try_new(22).expect("Failed to create ZstdLevel");
                Compression::ZSTD(zstd_level)
            }
        };

        props = props.set_compression(compression);
    }

    if let Some(encoding) = options.encoding {
        let encoding = match encoding {
            ParquetEncoding::PLAIN => Encoding::PLAIN,
            ParquetEncoding::RLE => Encoding::RLE,
            ParquetEncoding::DELTA_BINARY_PACKED => Encoding::DELTA_BINARY_PACKED,
            ParquetEncoding::DELTA_LENGTH_BYTE_ARRAY => Encoding::DELTA_LENGTH_BYTE_ARRAY,
            ParquetEncoding::DELTA_BYTE_ARRAY => Encoding::DELTA_BYTE_ARRAY,
            ParquetEncoding::RLE_DICTIONARY => Encoding::RLE_DICTIONARY,
        };

        props = props.set_encoding(encoding);
    }

    if let Some(size) = options.write_batch_size {
        props = props.set_write_batch_size(size);
    }

    if let Some(size) = options.data_pagesize_limit {
        props = props.set_data_page_size_limit(size);
    }

    if let Some(size) = options.dictionary_pagesize_limit {
        props = props.set_dictionary_page_size_limit(size);
    }

    if let Some(size) = options.dictionary_pagesize_limit {
        props = props.set_dictionary_page_size_limit(size);
    }

    if let Some(size) = options.max_row_group_size {
        props = props.set_max_row_group_size(size);
    }

    if let Some(created_by) = options.created_by {
        props = props.set_created_by(created_by);
    }


    let mut writer = ArrowWriter::try_new(output, reader.schema(), Some(props.build()))?;

    for batch in reader {
        match batch {
            Ok(batch) => writer.write(&batch)?,
            Err(error) => return Err(error.into()),
        }
    }

    match writer.close() {
        Ok(_) => Ok(()),
        Err(error) => Err(error),
    }
}


#[cfg(test)]
mod test_csv_to_parquet {
    use super::*;
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn should_convert_csv_to_parquet() {
        // Create a temporary CSV file with valid data
        let mut input_file = NamedTempFile::new().unwrap();
        let csv_data = "id,name\n1,Alice\n2,Bob";
        write!(input_file, "{}", csv_data).unwrap();

        let output_file = NamedTempFile::new().unwrap();

        let options = Options {
            input: input_file.path().to_path_buf(),
            output: output_file.path().to_path_buf(),
            schema_file: None,
            max_read_records: None,
            header: Some(true),
            delimiter: ',',
            compression: None,
            encoding: None,
            data_pagesize_limit: None,
            dictionary_pagesize_limit: None,
            write_batch_size: None,
            max_row_group_size: None,
            created_by: None,
            dictionary: false,
            statistics: None,
            print_schema: false,
            dry: false,
        };

        csv_to_parquet(options).unwrap();

        let output_file_size = output_file.path().metadata().expect("Failed to get file metadata").len();
        assert!(output_file_size > 0, "Output file is empty");

        let output_file_handle = File::open(output_file.path()).expect("Failed to open output file");
        let builder = ParquetRecordBatchReaderBuilder::try_new(output_file_handle).expect("Failed to create ParquetRecordBatchReaderBuilder");
        let mut reader = builder.build().expect("Failed to build ParquetRecordBatchReader");
        let output_content = reader.next().unwrap().expect("Failed to read record batch");
        assert_eq!(output_content.num_rows(), 2, "Number of rows in output file is incorrect");
        assert_eq!(output_content.column(0).as_any().downcast_ref::<arrow::array::Int64Array>().unwrap().values(), &[1, 2]);
        assert_eq!(output_content.column(1).as_any().downcast_ref::<arrow::array::StringArray>().unwrap().value(0), "Alice");
        assert_eq!(output_content.column(1).as_any().downcast_ref::<arrow::array::StringArray>().unwrap().value(1), "Bob");
    }
}