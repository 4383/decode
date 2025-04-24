mod parser;
mod engine;
mod tests;  // Add the tests module

use anyhow::{Context, Result, anyhow};
use clap::{Parser, ValueEnum};
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use serde_json::Value;

/// Decode - A high-performance query tool for JSON, YAML, and TOML data
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The query string
    #[arg(index = 1)]
    query: String,

    /// Input file (defaults to stdin if not provided)
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Input format (autodetected from file extension if not specified)
    #[arg(short = 'i', long, value_enum)]
    input_format: Option<InputFormat>,

    /// Output format
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Compact)]
    output: OutputFormat,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum InputFormat {
    /// JSON format
    Json,
    /// YAML format
    Yaml,
    /// TOML format
    Toml,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum OutputFormat {
    /// Pretty-printed JSON with indentation
    Pretty,
    /// Compact JSON (no extra whitespace)
    Compact,
    /// Raw output (for strings)
    Raw,
}

fn main() -> Result<()> {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Determine input format from file extension or explicit format flag
    let input_format = determine_input_format(&cli.file, cli.input_format)?;

    // Read the input data
    let input = read_input(&cli.file, input_format)?;

    // Parse the query
    let query = parser::parse_query(&cli.query)
        .context(format!("Failed to parse query: {}", cli.query))?;

    // Apply the query to the input data
    let result = engine::apply_query(&input, &query)
        .context("Failed to apply query")?;

    // Output the result in the requested format
    match cli.output {
        OutputFormat::Pretty => {
            println!("{}", serde_json::to_string_pretty(&result)
                .context("Failed to serialize result")?);
        },
        OutputFormat::Compact => {
            println!("{}", serde_json::to_string(&result)
                .context("Failed to serialize result")?);
        },
        OutputFormat::Raw => {
            // For raw output, if the result is a simple value, output it without quotes
            match result {
                Value::String(s) => println!("{}", s),
                Value::Number(n) => println!("{}", n),
                Value::Bool(b) => println!("{}", b),
                Value::Null => println!("null"),
                _ => println!("{}", serde_json::to_string(&result)
                    .context("Failed to serialize result")?),
            }
        },
    }

    Ok(())
}

/// Determine the input format from file extension or explicit format flag
fn determine_input_format(
    file_path: &Option<PathBuf>,
    explicit_format: Option<InputFormat>,
) -> Result<InputFormat> {
    // If format is explicitly specified, use that
    if let Some(format) = explicit_format {
        return Ok(format);
    }

    // If we have a file, try to determine format from extension
    if let Some(path) = file_path {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                match ext_str.to_lowercase().as_str() {
                    "json" => return Ok(InputFormat::Json),
                    "yml" | "yaml" => return Ok(InputFormat::Yaml),
                    "toml" => return Ok(InputFormat::Toml),
                    _ => {}
                }
            }
        }
    }

    // Default to JSON if format cannot be determined
    Ok(InputFormat::Json)
}

/// Read input data from file or stdin and parse according to the specified format
fn read_input(file_path: &Option<PathBuf>, format: InputFormat) -> Result<Value> {
    let input_text = if let Some(path) = file_path {
        // Read from file
        let mut file = File::open(path)
            .context(format!("Failed to open file: {}", path.display()))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .context("Failed to read file")?;
        content
    } else {
        // Read from stdin
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)
            .context("Failed to read from stdin")?;
        buffer
    };

    // Parse according to format
    match format {
        InputFormat::Json => {
            serde_json::from_str(&input_text)
                .context("Failed to parse JSON input")
        },
        InputFormat::Yaml => {
            serde_yaml::from_str(&input_text)
                .context("Failed to parse YAML input")
        },
        InputFormat::Toml => {
            let value = toml::from_str(&input_text)
                .context("Failed to parse TOML input")?;
            
            // Convert toml::Value to serde_json::Value for consistent querying
            toml_to_json_value(value)
        },
    }
}

/// Convert a TOML value to a JSON value
fn toml_to_json_value(toml_value: toml::Value) -> Result<Value> {
    match toml_value {
        toml::Value::String(s) => Ok(Value::String(s)),
        toml::Value::Integer(i) => Ok(Value::Number(serde_json::Number::from(i))),
        toml::Value::Float(f) => {
            // Convert float to Number (must be valid, non-NaN, non-infinite)
            if f.is_finite() {
                match serde_json::Number::from_f64(f) {
                    Some(n) => Ok(Value::Number(n)),
                    None => Err(anyhow!("Cannot represent float in JSON: {}", f)),
                }
            } else {
                Err(anyhow!("JSON cannot represent non-finite float: {}", f))
            }
        },
        toml::Value::Boolean(b) => Ok(Value::Bool(b)),
        toml::Value::Datetime(dt) => Ok(Value::String(dt.to_string())),
        toml::Value::Array(arr) => {
            let mut json_array = Vec::new();
            for item in arr {
                json_array.push(toml_to_json_value(item)?);
            }
            Ok(Value::Array(json_array))
        },
        toml::Value::Table(table) => {
            let mut map = serde_json::Map::new();
            for (key, value) in table {
                map.insert(key, toml_to_json_value(value)?);
            }
            Ok(Value::Object(map))
        }
    }
}
