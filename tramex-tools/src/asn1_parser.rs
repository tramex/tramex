//! ASN.1 Text Notation to JSON Parser
//! 
//! This module provides functionality to parse ASN.1 value notation (text format)
//! and convert it to JSON for easier manipulation and display.

#![allow(missing_docs)]

use pest::Parser;
use pest_derive::Parser;
use serde_json::{json, Value, Map};

/// ASN.1 Parser generated from pest grammar
#[derive(Parser)]
#[grammar = "asn1.pest"]
struct ASN1Parser;

/// Parse ASN.1 text notation and convert to JSON
/// 
/// # Arguments
/// * `input` - ASN.1 text notation string
/// 
/// # Returns
/// * `Ok(Value)` - Parsed JSON value
/// * `Err(String)` - Error message if parsing fails
/// 
/// # Example
/// ```
/// let asn1 = r#"{
///     cellIdentity '001234501'H,
///     trackingAreaCode '000065'H
/// }"#;
/// 
/// let json = parse_asn1_to_json(asn1)?;
/// println!("{}", serde_json::to_string_pretty(&json)?);
/// ```
pub fn parse_asn1_to_json(input: &str) -> Result<Value, String> {
    let pairs = ASN1Parser::parse(Rule::asn1_value, input)
        .map_err(|e| format!("ASN.1 parse error: {}", e))?;
    
    for pair in pairs {
        for inner_pair in pair.into_inner() {
            return parse_value(inner_pair);
        }
    }
    
    Err("No value found in ASN.1 input".to_string())
}

/// Parse a single ASN.1 value into JSON
fn parse_value(pair: pest::iterators::Pair<Rule>) -> Result<Value, String> {
    match pair.as_rule() {
        Rule::sequence => parse_sequence(pair),
        Rule::choice => parse_choice(pair),
        Rule::hex_string => Ok(parse_hex_string(pair.as_str())),
        Rule::bit_string => Ok(parse_bit_string(pair.as_str())),
        Rule::null_value => Ok(Value::Null),
        Rule::number => {
            let num_str = pair.as_str();
            if let Ok(num) = num_str.parse::<i64>() {
                Ok(json!(num))
            } else {
                Ok(Value::String(num_str.to_string()))
            }
        }
        Rule::identifier => Ok(Value::String(pair.as_str().to_string())),
        Rule::value => {
            // Unwrap nested value
            if let Some(inner) = pair.into_inner().next() {
                parse_value(inner)
            } else {
                Err("Empty value".to_string())
            }
        }
        _ => Err(format!("Unexpected rule: {:?}", pair.as_rule()))
    }
}

/// Parse ASN.1 sequence (object with fields or array)
fn parse_sequence(pair: pest::iterators::Pair<Rule>) -> Result<Value, String> {
    let mut map = Map::new();
    let mut array_items = Vec::new();
    let mut has_named_fields = false;
    let mut has_bare_values = false;
    
    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::element_list => {
                for element_pair in inner.into_inner() {
                    match element_pair.as_rule() {
                        Rule::element => {
                            // Check what's inside the element
                            let mut element_parts = element_pair.into_inner();
                            if let Some(first) = element_parts.next() {
                                match first.as_rule() {
                                    Rule::named_field => {
                                        // This is a named field
                                        has_named_fields = true;
                                        let mut field_parts = first.into_inner();
                                        if let (Some(key_pair), Some(value_pair)) = 
                                            (field_parts.next(), field_parts.next()) {
                                            let key = key_pair.as_str().to_string();
                                            let value = parse_value(value_pair)?;
                                            map.insert(key, value);
                                        }
                                    }
                                    _ => {
                                        // This is a bare value (array element)
                                        has_bare_values = true;
                                        let value = parse_value(first)?;
                                        array_items.push(value);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    
    // Decide if this is an object or array
    if has_bare_values && !has_named_fields {
        // Pure array
        Ok(Value::Array(array_items))
    } else if has_named_fields {
        // Object (possibly with some array items mixed in, but we prioritize object)
        Ok(Value::Object(map))
    } else {
        // Empty sequence
        Ok(Value::Object(Map::new()))
    }
}

/// Parse ASN.1 choice (tagged value)
fn parse_choice(pair: pest::iterators::Pair<Rule>) -> Result<Value, String> {
    let mut parts = pair.into_inner();
    
    if let (Some(tag), Some(value)) = (parts.next(), parts.next()) {
        let mut map = Map::new();
        map.insert(tag.as_str().to_string(), parse_value(value)?);
        Ok(Value::Object(map))
    } else {
        Err("Invalid choice format".to_string())
    }
}

/// Parse hex string and convert to readable format
fn parse_hex_string(s: &str) -> Value {
    // Remove quotes and 'H' suffix
    let hex = s.trim_start_matches('\'').trim_end_matches("'H");
    
    // Try to convert to decimal if it's a reasonable size
    if hex.len() <= 16 {
        if let Ok(num) = u64::from_str_radix(hex, 16) {
            return json!({
                "hex": hex,
                "decimal": num
            });
        }
    }
    
    // Otherwise just return the hex string
    Value::String(format!("0x{}", hex))
}

/// Parse bit string
fn parse_bit_string(s: &str) -> Value {
    // Remove quotes and 'B' suffix
    let bits = s.trim_start_matches('\'').trim_end_matches("'B");
    
    // Try to convert to decimal if it's a reasonable size
    if bits.len() <= 64 {
        if let Ok(num) = u64::from_str_radix(bits, 2) {
            return json!({
                "bits": bits,
                "decimal": num
            });
        }
    }
    
    // Otherwise just return the bit string
    Value::String(format!("0b{}", bits))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_sequence() {
        let asn1 = r#"{
            q-RxLevMin -70,
            q-QualMin -20
        }"#;
        
        let result = parse_asn1_to_json(asn1);
        assert!(result.is_ok());
        
        let json = result.unwrap();
        assert_eq!(json["q-RxLevMin"], -70);
        assert_eq!(json["q-QualMin"], -20);
    }

    #[test]
    fn test_hex_string() {
        let asn1 = r#"{
            trackingAreaCode '000065'H,
            cellIdentity '001234501'H
        }"#;
        
        let result = parse_asn1_to_json(asn1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nested_sequence() {
        let asn1 = r#"{
            cellSelectionInfo {
                q-RxLevMin -70,
                q-QualMin -20
            }
        }"#;
        
        let result = parse_asn1_to_json(asn1);
        assert!(result.is_ok());
        
        let json = result.unwrap();
        assert!(json["cellSelectionInfo"].is_object());
    }

    #[test]
    fn test_array() {
        let asn1 = r#"{
            {
                0,
                0,
                1
            }
        }"#;
        
        let result = parse_asn1_to_json(asn1);
        assert!(result.is_ok());
        
        let json = result.unwrap();
        assert!(json.is_array());
    }

    #[test]
    fn test_choice() {
        let asn1 = r#"{
            c1: systemInformationBlockType1: {
                cellSelectionInfo {
                    q-RxLevMin -70
                }
            }
        }"#;
        
        let result = parse_asn1_to_json(asn1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_null_value() {
        let asn1 = r#"{
            someField NULL
        }"#;
        
        let result = parse_asn1_to_json(asn1);
        assert!(result.is_ok());
        
        let json = result.unwrap();
        assert!(json["someField"].is_null());
    }

    #[test]
    fn test_bit_string() {
        let asn1 = r#"{
            monitoringSymbolsWithinSlot '10000000000000'B
        }"#;
        
        let result = parse_asn1_to_json(asn1);
        assert!(result.is_ok());
    }
}
