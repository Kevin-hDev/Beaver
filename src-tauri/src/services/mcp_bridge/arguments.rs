use serde_json::Value;
use std::io::{self, Write};

const MAX_ARGUMENT_BYTES: usize = 65_536;
const MAX_DEPTH: usize = 16;
const MAX_ELEMENTS: usize = 256;
const INVALID: &str = "arguments MCP invalides";

pub fn validate(arguments: &Value, schema: Option<&Value>) -> Result<(), String> {
    if !arguments.is_object() {
        return Err(INVALID.to_string());
    }
    let mut elements = 0;
    check_structure(arguments, 0, &mut elements)?;
    serde_json::to_writer(LimitedWriter::new(MAX_ARGUMENT_BYTES), arguments)
        .map_err(|_| INVALID.to_string())?;
    let schema = schema.ok_or_else(|| INVALID.to_string())?;
    super::schema::validate(schema, arguments).map_err(|_| INVALID.to_string())
}

fn check_structure(value: &Value, depth: usize, elements: &mut usize) -> Result<(), String> {
    if depth > MAX_DEPTH {
        return Err(INVALID.to_string());
    }
    match value {
        Value::Object(map) => {
            *elements = elements.saturating_add(map.len());
            if *elements > MAX_ELEMENTS {
                return Err(INVALID.to_string());
            }
            for value in map.values() {
                check_structure(value, depth + 1, elements)?;
            }
        }
        Value::Array(items) => {
            *elements = elements.saturating_add(items.len());
            if *elements > MAX_ELEMENTS {
                return Err(INVALID.to_string());
            }
            for value in items {
                check_structure(value, depth + 1, elements)?;
            }
        }
        _ => {}
    }
    Ok(())
}

struct LimitedWriter {
    written: usize,
    limit: usize,
}

impl LimitedWriter {
    fn new(limit: usize) -> Self {
        Self { written: 0, limit }
    }
}

impl Write for LimitedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.written.saturating_add(bytes.len()) > self.limit {
            return Err(io::Error::other("limit"));
        }
        self.written += bytes.len();
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
