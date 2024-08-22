use clap::{Arg, Command};
use image::{ImageFormat};
use serde_json::Value;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use csv::StringRecord;

fn main() {
    let matches = Command::new("file_converter")
        .version("1.0")
        .author("Fernando Alzueta <devfernandoa@gmail.com>")
        .about("Converte arquivos de um formato para outro")
        .arg(
            Arg::new("input")
                .help("Arquivo de entrada")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("format")
                .help("Formato de saída (json, csv, tsv, xml, png, jpg, gif)")
                .required(true)
                .index(2),
        )
        .get_matches();

    let input_file = matches.value_of("input").unwrap();
    let format = matches.value_of("format").unwrap();

    let output_file = generate_output_filename(input_file, format);

    if format == "png" || format == "jpg" || format == "gif" {
        match is_valid_image(input_file) {
            Ok(_) => {
                convert_image(input_file, &output_file, format)
                    .expect("Falha ao converter a imagem");
            }
            Err(e) => {
                eprintln!("O arquivo de entrada não é uma imagem válida: {}", e);
            }
        }
    } else {
        let data = if input_file.ends_with(".csv") {
            read_csv(input_file).expect("Falha ao ler o arquivo CSV de entrada")
        } else if input_file.ends_with(".tsv") {
            read_tsv(input_file).expect("Falha ao ler o arquivo TSV de entrada")
        } else if input_file.ends_with(".xml") {
            read_xml(input_file).expect("Falha ao ler o arquivo XML de entrada")
        } else {
            read_json(input_file).expect("Falha ao ler o arquivo de entrada")
        };

        match format {
            "json" => write_json(&output_file, &data).expect("Falha ao escrever JSON"),
            "csv" => write_csv(&output_file, &data).expect("Falha ao escrever CSV"),
            "tsv" => write_tsv(&output_file, &data).expect("Falha ao escrever TSV"),
            "xml" => write_xml(&output_file, &data).expect("Falha ao escrever XML"),
            _ => eprintln!("Formato não suportado: {}", format),
        }
    }
}

fn generate_output_filename(input_file: &str, format: &str) -> String {
    let path = Path::new(input_file);
    let stem = path.file_stem().unwrap().to_str().unwrap();
    format!("{}.{}", stem, format)
}

fn is_valid_image(input_file: &str) -> Result<(), String> {
    match image::open(input_file) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Erro ao abrir a imagem: {}", e)),
    }
}

fn convert_image(input_file: &str, output_file: &str, format: &str) -> io::Result<()> {
    let img = image::open(input_file).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let output_format = match format {
        "png" => ImageFormat::Png,
        "jpg" => ImageFormat::Jpeg,
        "gif" => ImageFormat::Gif,
        _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Formato de imagem não suportado")),
    };

    img.save_with_format(output_file, output_format).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(())
}

fn read_json(filename: &str) -> io::Result<Value> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    serde_json::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn read_csv(filename: &str) -> io::Result<Value> {
    let mut rdr = csv::Reader::from_path(filename)?;
    let headers = rdr.headers()?.clone();
    let mut records = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        records.push(record);
    }
    Ok(csv_to_json(headers, records))
}

fn read_tsv(filename: &str) -> io::Result<Value> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_path(filename)?;
    let headers = rdr.headers()?.clone();
    let mut records = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        records.push(record);
    }
    Ok(csv_to_json(headers, records))
}

fn read_xml(filename: &str) -> io::Result<Value> {
    let mut file = File::open(filename)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    serde_xml_rs::from_str(&contents).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn write_json(filename: &str, data: &Value) -> io::Result<()> {
    let file = File::create(filename)?;
    serde_json::to_writer(file, data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn write_csv(filename: &str, data: &Value) -> io::Result<()> {
    let mut wtr = csv::WriterBuilder::new()
        .quote_style(csv::QuoteStyle::Always)
        .from_path(filename)?;
    if let Value::Array(records) = data {
        for record in records {
            if let Value::Object(map) = record {
                let string_fields: Vec<String> = map.values().map(|v| v.as_str().unwrap_or("").to_string()).collect();
                wtr.write_record(&string_fields).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            }
        }
    }
    Ok(())
}

fn write_tsv(filename: &str, data: &Value) -> io::Result<()> {
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .quote_style(csv::QuoteStyle::Always)
        .from_path(filename)?;
    if let Value::Array(records) = data {
        for record in records {
            if let Value::Object(map) = record {
                let string_fields: Vec<String> = map.values().map(|v| v.as_str().unwrap_or("").to_string()).collect();
                wtr.write_record(&string_fields).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            }
        }
    }
    Ok(())
}

fn write_xml(filename: &str, data: &Value) -> io::Result<()> {
    let file = File::create(filename)?;
    serde_xml_rs::to_writer(file, data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn csv_to_json(headers: StringRecord, records: Vec<StringRecord>) -> Value {
    let mut json_records = Vec::new();
    for record in records {
        let mut json_record = serde_json::Map::new();
        for (header, field) in headers.iter().zip(record.iter()) {
            json_record.insert(header.to_string(), Value::String(field.to_string()));
        }
        json_records.push(Value::Object(json_record));
    }
    Value::Array(json_records)
}