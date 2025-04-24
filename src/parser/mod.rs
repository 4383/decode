use anyhow::{Context, Result};
use pest::Parser;
use pest_derive::Parser;
use pest::iterators::{Pair, Pairs};

#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub struct QueryParser;

/// Represents different types of path segments in a JSON query
#[derive(Debug, Clone)]
pub enum PathSegment {
    Field(String),
    Index(i64),
    MultiIndex(Vec<i64>),
    Filter(FilterExpression),
    RecursiveWildcard,
}

/// Represents a filter expression with path, operator and value
#[derive(Debug, Clone)]
pub struct FilterExpression {
    pub path: Vec<PathSegment>,
    pub operator: ComparisonOperator,
    pub value: LiteralValue,
}

/// Comparison operators supported in filter expressions
#[derive(Debug, Clone)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

/// Literal values that can be compared against in filters
#[derive(Debug, Clone)]
pub enum LiteralValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Null,
}

/// Represents a complete JSON query
#[derive(Debug)]
pub struct Query {
    pub path_segments: Vec<PathSegment>,
    pub recursive_paths: Vec<Vec<PathSegment>>,
}

/// Parses a query string into a structured Query object
pub fn parse_query(input: &str) -> Result<Query> {
    // Parse the input using the pest parser
    let pairs = QueryParser::parse(Rule::query, input)
        .context(format!("Failed to parse query: {}", input))?
        .next()
        .context("Empty parse result")?
        .into_inner();

    let mut path_segments = Vec::new();
    let mut recursive_paths = Vec::new();
    
    for pair in pairs {
        match pair.as_rule() {
            Rule::root => {
                // Root is handled implicitly
            },
            Rule::path => {
                // Process the main path segments
                path_segments.extend(parse_path_segments(pair.into_inner())?);
            },
            Rule::recursive_descent => {
                // Process recursive descent
                let mut inner_pairs = pair.into_inner();
                let field_accessor = inner_pairs.next().context("Expected field accessor after ..")?;
                let field_name = field_accessor.as_str();
                recursive_paths.push(vec![PathSegment::Field(field_name.to_string())]);
            },
            _ => {}
        }
    }

    Ok(Query {
        path_segments,
        recursive_paths,
    })
}

/// Parse a path into individual path segments
fn parse_path_segments(pairs: Pairs<Rule>) -> Result<Vec<PathSegment>> {
    let mut segments = Vec::new();
    
    for pair in pairs {
        match pair.as_rule() {
            Rule::dot_field => {
                let field_name = pair.into_inner()
                    .next()
                    .context("Expected field name after dot")?
                    .as_str()
                    .to_string();
                segments.push(PathSegment::Field(field_name));
            },
            Rule::bracket_access => {
                let inner = pair.into_inner().next().context("Expected content inside brackets")?;
                match inner.as_rule() {
                    Rule::integer => {
                        // Numeric index
                        let idx = inner.as_str().parse::<i64>()
                            .context(format!("Failed to parse index: {}", inner.as_str()))?;
                        segments.push(PathSegment::Index(idx));
                    },
                    Rule::string => {
                        // String index (treated as field name)
                        let s = inner.as_str();
                        let field_name = s[1..s.len()-1].to_string();
                        segments.push(PathSegment::Field(field_name));
                    },
                    Rule::multi_index => {
                        // Multiple indices
                        let mut indices = Vec::new();
                        for idx_pair in inner.into_inner() {
                            let idx = idx_pair.as_str().parse::<i64>()
                                .context(format!("Failed to parse index: {}", idx_pair.as_str()))?;
                            indices.push(idx);
                        }
                        segments.push(PathSegment::MultiIndex(indices));
                    },
                    _ => return Err(anyhow::anyhow!("Unexpected bracket content: {:?}", inner.as_rule())),
                }
            },
            Rule::filter => {
                let filter_expr_pair = pair.into_inner().next()
                    .context("Expected filter expression")?;
                let expr = parse_filter_expression(filter_expr_pair)?;
                segments.push(PathSegment::Filter(expr));
            },
            Rule::recursive_wildcard => {
                segments.push(PathSegment::RecursiveWildcard);
            },
            _ => {}
        }
    }
    
    Ok(segments)
}

/// Parse a filter expression into a FilterExpression
fn parse_filter_expression(pair: Pair<Rule>) -> Result<FilterExpression> {
    // The pair should either be filter_expr directly or contain it
    let filter_expr_pair = if pair.as_rule() == Rule::filter_expr {
        pair
    } else {
        pair.into_inner().next().context("Expected filter expression")?
    };
    
    let mut inner_pairs = filter_expr_pair.into_inner();
    
    // Parse the path part (@.path.to.field)
    let filter_path_pair = inner_pairs.next().context("Expected filter path")?;
    let mut path = Vec::new();
    
    // Process the filter path after @
    for path_pair in filter_path_pair.into_inner() {
        match path_pair.as_rule() {
            Rule::dot_field => {
                let field_name = path_pair.into_inner()
                    .next()
                    .context("Expected field name after dot")?
                    .as_str()
                    .to_string();
                path.push(PathSegment::Field(field_name));
            },
            Rule::bracket_access => {
                // Handle bracket access in filters
                let inner = path_pair.into_inner().next().context("Expected content inside brackets")?;
                match inner.as_rule() {
                    Rule::integer => {
                        let idx = inner.as_str().parse::<i64>()
                            .context(format!("Failed to parse index: {}", inner.as_str()))?;
                        path.push(PathSegment::Index(idx));
                    },
                    Rule::string => {
                        let s = inner.as_str();
                        let field_name = s[1..s.len()-1].to_string();
                        path.push(PathSegment::Field(field_name));
                    },
                    _ => {}, // Ignore other types for now
                }
            },
            _ => {},
        }
    }
    
    // Parse the operator
    let comparator = inner_pairs.next().context("Expected comparison operator")?;
    let op_str = comparator.as_str();
    let operator = match op_str {
        "==" => ComparisonOperator::Equal,
        "!=" => ComparisonOperator::NotEqual,
        ">" => ComparisonOperator::GreaterThan,
        ">=" => ComparisonOperator::GreaterThanOrEqual,
        "<" => ComparisonOperator::LessThan,
        "<=" => ComparisonOperator::LessThanOrEqual,
        _ => return Err(anyhow::anyhow!("Unsupported comparison operator: {}", op_str)),
    };
    
    // Parse the literal value
    let literal_pair = inner_pairs.next().context("Expected literal in filter expression")?;
    
    let value = match literal_pair.as_rule() {
        Rule::string => {
            let s = literal_pair.as_str();
            // Remove the quotes
            LiteralValue::String(s[1..s.len()-1].to_string())
        },
        Rule::integer => {
            let i = literal_pair.as_str().parse::<i64>()
                .context(format!("Failed to parse integer: {}", literal_pair.as_str()))?;
            LiteralValue::Integer(i)
        },
        Rule::boolean => {
            let b = literal_pair.as_str() == "true";
            LiteralValue::Boolean(b)
        },
        Rule::null => LiteralValue::Null,
        _ => {
            // If it's the literal rule, we need to go one level deeper
            if let Some(inner) = literal_pair.into_inner().next() {
                match inner.as_rule() {
                    Rule::string => {
                        let s = inner.as_str();
                        LiteralValue::String(s[1..s.len()-1].to_string())
                    },
                    Rule::integer => {
                        let i = inner.as_str().parse::<i64>()
                            .context(format!("Failed to parse integer: {}", inner.as_str()))?;
                        LiteralValue::Integer(i)
                    },
                    Rule::boolean => {
                        let b = inner.as_str() == "true";
                        LiteralValue::Boolean(b)
                    },
                    Rule::null => LiteralValue::Null,
                    _ => return Err(anyhow::anyhow!("Unsupported inner literal type: {:?}", inner.as_rule())),
                }
            } else {
                return Err(anyhow::anyhow!("Empty literal value"));
            }
        }
    };
    
    Ok(FilterExpression {
        path,
        operator,
        value,
    })
}