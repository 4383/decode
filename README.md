# Decode

[![Rust](https://github.com/4383/decode/actions/workflows/rust.yml/badge.svg)](https://github.com/4383/decode/actions/workflows/rust.yml)
![Crates.io Version](https://img.shields.io/crates/v/decode)
![Crates.io Total Downloads](https://img.shields.io/crates/d/decode)


**A High-Performance Query Tool for JSON, YAML and TOML**

`decode` is a command-line tool written in Rust that allows you to extract and transform data from JSON, YAML, and TOML using a powerful, JSONPath-like query syntax.

## Features

- **Fast**: Built with Rust for high performance on large data files
- **Multi-format**: Support for JSON, YAML, and TOML input formats
- **Intuitive Syntax**: Familiar JSONPath-like query language
- **Powerful Selectors**: Support for complex queries and filters
- **Multiple Output Formats**: Choose between pretty, compact, or raw output

## Installation

### From crates.io

```bash
cargo install decode
```

### From Source

```bash
# Clone the repository
git clone https://github.com/4383/decode.git
cd decode

# Build with Cargo
cargo build --release

# The binary will be available at ./target/release/decode
```

## Usage

```bash
decode <QUERY> [OPTIONS]
```

### Options

- `-f, --file <FILE>`: Input file path (reads from stdin if not provided)
- `-i, --input-format <FORMAT>`: Input format [possible values: json, yaml, toml] (autodetected from file extension if not specified)
- `-o, --output <FORMAT>`: Output format [default: compact] [possible values: pretty, compact, raw]

## Query Syntax

### Basic Path Expressions

- `$.name` - Access a field named "name"
- `$.users[0]` - Access the first element of the "users" array
- `$.users[-1]` - Access the last element of the "users" array
- `$.users[0,2,4]` - Access multiple indices (returns an array)
- `$.users[0].name.first` - Chain paths to access nested data

### Filter Expressions

- `$.users[?(@.age > 30)]` - Filter users with age greater than 30
- `$.items[?(@.price < 10.0)]` - Filter items with price less than 10.0
- `$.users[?(@.email == null)]` - Filter users with email explicitly set to null

### Wildcards and Recursive Descent

- `$.store[*]` - Get all values in the "store" object
- `$..title` - Find all "title" fields at any depth in the document

### Complete Grammar Reference

#### Root Selectors
- `$` - Represents the root of the document
- `root` - Alternative syntax for the root

#### Field Access
- `.fieldName` - Access a field by name
- `."field name"` - Access a field with spaces or special characters

#### Array Access
- `[n]` - Access array element at index n (zero-based)
- `[-n]` - Access nth element from the end of the array
- `[m,n,p]` - Access multiple specific indices, returning an array

#### Recursive Operators
- `..field` - Deep scan for all occurrences of "field" at any level
- `[*]` - Select all elements/properties of an array/object

#### Filter Expressions
- `[?(<expression>)]` - Filter elements based on a condition

#### Filter Operators
- `==` - Equal to
- `!=` - Not equal to
- `>` - Greater than
- `>=` - Greater than or equal to
- `<` - Less than
- `<=` - Less than or equal to

#### Filter Values
- `"value"` - String literal (must be quoted)
- `123` - Integer literal
- `true` / `false` - Boolean literals
- `null` - Null literal

## CLI Examples

### Basic Usage Examples

**Extract a specific field:**
```bash
# Get the store name
decode '$.store.name' -f sample-data.json
```

**Access nested properties:**
```bash
# Get store location
decode '$.store.location' -f sample-data.json -o pretty

# Get a specific deeply nested value
decode '$.store.location.city' -f sample-data.json
```

**Access array elements:**
```bash
# Get the first book in the store
decode '$.store.book[0]' -f sample-data.json -o pretty

# Get the last book using negative index
decode '$.store.book[-1]' -f sample-data.json -o pretty

# Get specific elements from an array (first and third)
decode '$.store.book[0,2]' -f sample-data.json -o pretty
```

### Format-specific Examples

**Query a YAML configuration file:**
```bash
# Extract the server host from a YAML config
decode '$.server.host' -f config.yaml

# Format is automatically detected from file extension
```

**Query a TOML file:**
```bash
# Get dependencies from Cargo.toml
decode '$.dependencies' -f Cargo.toml -o pretty

# Explicitly specify input format
decode '$.package.name' --file Cargo.toml --input-format toml
```

**Mix and match formats:**
```bash
# Read from YAML, output as compact JSON
cat config.yaml | decode '$.database' --input-format yaml

# Extract specific fields from TOML configuration
decode '$.tool.decode.keywords' -f pyproject.toml -i toml
```

### Filter Expressions Examples

**Filter by numeric comparison:**
```bash
# Find all books with price over 15
decode '$.store.book[?(@.price > 15)]' -f sample-data.json -o pretty

# Find customers aged 30 or younger
decode '$.customers[?(@.age <= 30)]' -f sample-data.json -o pretty
```

**Filter by equality:**
```bash
# Find books that are in stock
decode '$.store.book[?(@.inStock == true)]' -f sample-data.json -o pretty

# Find products with null specifications
decode '$.store.electronics[?(@.specifications == null)]' -f sample-data.json -o pretty
```

### Recursive Search Examples

**Find all titles in the document:**
```bash
# Get all title fields at any level in the document
decode '$..title' -f sample-data.json
```

**Find all price values across the document:**
```bash
# Get all price fields from anywhere in the document
decode '$..price' -f sample-data.json
```

### Output Format Examples

**Pretty print (formatted JSON):**
```bash
# Output nicely formatted, indented JSON
decode '$.customers[0]' -f sample-data.json -o pretty
```

**Compact output (default):**
```bash
# Output JSON without extra whitespace
decode '$.customers[0]' -f sample-data.json -o compact
```

**Raw output (for simple values):**
```bash
# Output raw value without quotes for strings
decode '$.store.name' -f sample-data.json -o raw
```

### Reading from stdin

**Process JSON from stdin:**
```bash
# Pipe JSON input to decode
cat sample-data.json | decode '$.store.name'

# Use output from another command
curl -s https://api.example.com/data | decode '$.results[0].id'
```

### Advanced Examples

**Combine multiple techniques:**
```bash
# Get names of books that are out of stock
decode '$.store.book[?(@.inStock == false)].title' -f sample-data.json

# Find customers who are active and over 30
decode '$.customers[?(@.active == true && @.age > 30)]' -f sample-data.json -o pretty

# Get all items that need restocking
decode '$.statistics.inventory.needRestock' -f sample-data.json
```

## Error Handling

`decode` provides informative error messages when:
- The JSON file cannot be read or parsed
- The query syntax is invalid
- A field or array index does not exist in the JSON

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request