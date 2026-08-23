use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use super::state::{
    ProjectionState, RootField, ScannedProjection, MAX_CAPTURE_ITEMS, MAX_KEY_BYTES,
    MAX_MODEL_BYTES, MAX_SCAN_DEPTH, MAX_TIER_BYTES, MAX_TYPE_BYTES,
};
use super::{lex, string, ScanError};

pub(super) fn parse(body: &[u8]) -> Result<ScannedProjection, ScanError> {
    parse_with_hook(body, None)
}

pub(super) fn parse_with_key_zeroize_hook(
    body: &[u8],
    key_zeroized: Arc<AtomicBool>,
) -> Result<ScannedProjection, ScanError> {
    parse_with_hook(body, Some(key_zeroized))
}

fn parse_with_hook(
    body: &[u8],
    key_zeroized: Option<Arc<AtomicBool>>,
) -> Result<ScannedProjection, ScanError> {
    std::str::from_utf8(body).map_err(|_| ScanError)?;
    let mut scanner = JsonScanner {
        body,
        cursor: 0,
        state: ProjectionState::default(),
        key_zeroized,
    };
    scanner.whitespace();
    scanner.object(0, true)?;
    scanner.whitespace();
    if scanner.cursor != body.len() {
        return Err(ScanError);
    }
    scanner.state.finish()
}

struct JsonScanner<'body> {
    body: &'body [u8],
    cursor: usize,
    state: ProjectionState,
    key_zeroized: Option<Arc<AtomicBool>>,
}

impl JsonScanner<'_> {
    fn value(&mut self, depth: usize) -> Result<(), ScanError> {
        self.depth(depth)?;
        self.whitespace();
        match self.body.get(self.cursor).copied() {
            Some(b'{') => self.object(depth, false),
            Some(b'[') => self.array(depth),
            Some(b'"') => string::skip(self.body, &mut self.cursor),
            Some(b't') => lex::literal(self.body, &mut self.cursor, b"true"),
            Some(b'f') => lex::literal(self.body, &mut self.cursor, b"false"),
            Some(b'n') => lex::literal(self.body, &mut self.cursor, b"null"),
            Some(b'-' | b'0'..=b'9') => lex::number(self.body, &mut self.cursor),
            _ => Err(ScanError),
        }
    }

    fn object(&mut self, depth: usize, root: bool) -> Result<(), ScanError> {
        self.depth(depth)?;
        lex::byte(self.body, &mut self.cursor, b'{')?;
        self.whitespace();
        if lex::optional_byte(self.body, &mut self.cursor, b'}') {
            return Ok(());
        }
        loop {
            self.whitespace();
            let mut key = string::decode_with_zeroize_hook(
                self.body,
                &mut self.cursor,
                MAX_KEY_BYTES,
                self.key_zeroized.as_ref().map(Arc::clone),
            )?;
            let field = self.state.observe_key(key.as_str()?, root);
            // Les clés peuvent elles-mêmes être sensibles : elles sont effacées avant la valeur.
            key.erase();
            self.whitespace();
            lex::byte(self.body, &mut self.cursor, b':')?;
            self.state.count_element()?;
            if root {
                self.state.claim(field)?;
                self.root_value(field, depth.saturating_add(1))?;
            } else {
                self.value(depth.saturating_add(1))?;
            }
            self.whitespace();
            match self.body.get(self.cursor).copied() {
                Some(b',') => self.cursor += 1,
                Some(b'}') => {
                    self.cursor += 1;
                    return Ok(());
                }
                _ => return Err(ScanError),
            }
        }
    }

    fn array(&mut self, depth: usize) -> Result<(), ScanError> {
        self.depth(depth)?;
        lex::byte(self.body, &mut self.cursor, b'[')?;
        self.whitespace();
        if lex::optional_byte(self.body, &mut self.cursor, b']') {
            return Ok(());
        }
        loop {
            self.state.count_element()?;
            self.value(depth.saturating_add(1))?;
            self.whitespace();
            match self.body.get(self.cursor).copied() {
                Some(b',') => self.cursor += 1,
                Some(b']') => {
                    self.cursor += 1;
                    return Ok(());
                }
                _ => return Err(ScanError),
            }
        }
    }

    fn root_value(&mut self, field: RootField, depth: usize) -> Result<(), ScanError> {
        self.depth(depth)?;
        self.whitespace();
        match field {
            RootField::Model => {
                let value = self.identifier(MAX_MODEL_BYTES)?;
                self.state.set_model(value)
            }
            RootField::ServiceTier => {
                let value = self.optional_identifier(MAX_TIER_BYTES)?;
                self.state.set_service_tier(value)
            }
            RootField::EnvelopeType => {
                let value = self.optional_identifier(MAX_TYPE_BYTES)?;
                self.state.set_envelope_type(value)
            }
            RootField::Input => {
                let count = self.counted_array(depth)?;
                self.state.set_input_count(count);
                Ok(())
            }
            RootField::Tools => {
                let count = self.counted_array(depth)?;
                self.state.set_tool_count(count);
                Ok(())
            }
            RootField::Unknown => self.value(depth),
        }
    }

    fn counted_array(&mut self, depth: usize) -> Result<usize, ScanError> {
        lex::byte(self.body, &mut self.cursor, b'[')?;
        self.whitespace();
        if lex::optional_byte(self.body, &mut self.cursor, b']') {
            return Ok(0);
        }
        let mut count = 0_usize;
        loop {
            if count >= MAX_CAPTURE_ITEMS {
                return Err(ScanError);
            }
            count += 1;
            self.state.count_element()?;
            self.value(depth.saturating_add(1))?;
            self.whitespace();
            match self.body.get(self.cursor).copied() {
                Some(b',') => self.cursor += 1,
                Some(b']') => {
                    self.cursor += 1;
                    return Ok(count);
                }
                _ => return Err(ScanError),
            }
        }
    }

    fn identifier(&mut self, max_bytes: usize) -> Result<string::DecodedString, ScanError> {
        self.whitespace();
        string::decode(self.body, &mut self.cursor, max_bytes)
    }

    fn optional_identifier(
        &mut self,
        max_bytes: usize,
    ) -> Result<Option<string::DecodedString>, ScanError> {
        self.whitespace();
        match self.body.get(self.cursor).copied() {
            Some(b'"') => self.identifier(max_bytes).map(Some),
            Some(b'n') => {
                lex::literal(self.body, &mut self.cursor, b"null")?;
                Ok(None)
            }
            _ => Err(ScanError),
        }
    }

    fn whitespace(&mut self) {
        lex::whitespace(self.body, &mut self.cursor);
    }

    fn depth(&self, depth: usize) -> Result<(), ScanError> {
        (depth <= MAX_SCAN_DEPTH).then_some(()).ok_or(ScanError)
    }
}
