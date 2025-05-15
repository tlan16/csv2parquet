use arrow_schema::Schema;
use std::fs::File;
use std::io::BufReader;

pub fn read_schema_from_schema_file_in_json(file_path: String) -> Schema {
    // Open the file in read-only mode.
    let file = File::open(&file_path)
        .map_err(|e| format!("Failed to open file {:?}: {}", file_path, e))
        .expect("Failed to open file");

    // Create a buffered reader for efficiency.
    let reader = BufReader::new(file);

    // Deserialize the JSON directly into an Arrow Schema.
    let schema: Schema = serde_json::from_reader(reader)
        .map_err(|e| format!("Failed to parse JSON as Arrow Schema: {}", e))
        .expect("Failed to parse JSON");

    schema
}

pub fn infer_scheam_from_data_file(
    file_path: String,
    delimiter: u8,
    max_read_records: Option<usize>,
    has_header: bool,
) -> Schema {
    let schema = arrow_csv::reader::infer_schema_from_files(
        &[file_path],
        delimiter,
        max_read_records,
        has_header,
    )
    .expect("Failed to infer schema from data file");
    schema
}

#[cfg(test)]
mod test_read_schema_from_json_file {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn from_json_file_valid() {
        // Create a temporary file with a valid Arrow Schema JSON
        let mut temp_file = NamedTempFile::new().unwrap();
        let schema_json = r#"
            {
              "fields": [
                {
                  "name": "id",
                  "data_type": "Int32",
                  "nullable": false,
                  "dict_id": 0,
                  "dict_is_ordered": false,
                  "metadata": {}
                },
                {
                  "name": "name",
                  "data_type": "Utf8",
                  "nullable": true,
                  "dict_id": 0,
                  "dict_is_ordered": false,
                  "metadata": {}
                }
              ],
              "metadata": {}
            }
        "#;
        write!(temp_file, "{}", schema_json).unwrap();

        let actual = read_schema_from_schema_file_in_json(temp_file.path().to_str().unwrap().to_string());
        let expected = Schema::new(vec![
            arrow_schema::Field::new("id", arrow_schema::DataType::Int32, false),
            arrow_schema::Field::new("name", arrow_schema::DataType::Utf8, true),
        ]);
        assert_eq!(actual.fields, expected.fields);
    }

    #[test]
    #[should_panic(expected = "Failed to open file")]
    fn from_json_file_file_not_found() {
        read_schema_from_schema_file_in_json("non_existent_file.json".to_string());
    }

    #[test]
    #[should_panic(expected = "Failed to parse JSON")]
    fn from_json_file_invalid_json() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "not a json").unwrap();
        read_schema_from_schema_file_in_json(temp_file.path().to_str().unwrap().to_string());
    }
}

#[cfg(test)]
mod test_infer_schema_from_data_file {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn from_data_file_valid() {
        // Create a temporary CSV file with valid data
        let mut temp_file = NamedTempFile::new().unwrap();
        let csv_data = "id,name\n1,Alice\n2,Bob";
        write!(temp_file, "{}", csv_data).unwrap();

        // Should not panic
        let actual = infer_scheam_from_data_file(
            temp_file.path().to_str().unwrap().to_string(),
            b',',
            Some(10),
            true,
        );
        let expected = Schema::new(vec![
            arrow_schema::Field::new("id", arrow_schema::DataType::Int64, true),
            arrow_schema::Field::new("name", arrow_schema::DataType::Utf8, true),
        ]);
        assert_eq!(actual.fields, expected.fields);
    }
}