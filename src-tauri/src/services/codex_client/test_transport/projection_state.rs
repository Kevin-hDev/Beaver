use super::string::DecodedString;
use super::ScanError;

pub(super) const MAX_CAPTURE_ITEMS: usize = 2_048;
pub(super) const MAX_MODEL_BYTES: usize = 128;
pub(super) const MAX_TIER_BYTES: usize = 16;
pub(super) const MAX_TYPE_BYTES: usize = 32;
pub(super) const MAX_SCAN_DEPTH: usize = 32;
pub(super) const MAX_SCAN_ELEMENTS: usize = 16_384;
pub(super) const MAX_KEY_BYTES: usize = 256;

#[derive(Clone, Copy)]
pub(super) enum RootField {
    Model,
    Reasoning,
    Effort,
    ServiceTier,
    EnvelopeType,
    Input,
    Tools,
    Unknown,
}

#[derive(Default)]
pub(super) struct ProjectionState {
    elements: usize,
    forbidden_field_present: bool,
    model: Option<String>,
    reasoning_effort: Option<String>,
    seen_reasoning: bool,
    seen_effort: bool,
    service_tier: Option<String>,
    envelope_type: Option<String>,
    input_count: usize,
    tool_count: usize,
    seen_model: bool,
    seen_service_tier: bool,
    seen_envelope_type: bool,
    seen_input: bool,
    seen_tools: bool,
}

pub(super) struct ScannedProjection {
    pub(super) model: String,
    pub(super) reasoning_effort: Option<String>,
    pub(super) service_tier: Option<String>,
    pub(super) envelope_type: Option<String>,
    pub(super) input_count: usize,
    pub(super) tool_count: usize,
    pub(super) forbidden_field_present: bool,
}

impl ProjectionState {
    pub(super) fn observe_key(&mut self, key: &str, root: bool, reasoning: bool) -> RootField {
        self.forbidden_field_present |=
            matches!(key, "access_token" | "refresh_token" | "authorization");
        if !root {
            return if reasoning && key == "effort" {
                RootField::Effort
            } else {
                RootField::Unknown
            };
        }
        match key {
            "model" => RootField::Model,
            "reasoning" => RootField::Reasoning,
            "service_tier" => RootField::ServiceTier,
            "type" => RootField::EnvelopeType,
            "input" => RootField::Input,
            "tools" => RootField::Tools,
            _ => RootField::Unknown,
        }
    }

    pub(super) fn count_element(&mut self) -> Result<(), ScanError> {
        if self.elements >= MAX_SCAN_ELEMENTS {
            return Err(ScanError);
        }
        self.elements += 1;
        Ok(())
    }

    pub(super) fn claim(&mut self, field: RootField) -> Result<(), ScanError> {
        let seen = match field {
            RootField::Model => &mut self.seen_model,
            RootField::Reasoning => &mut self.seen_reasoning,
            RootField::Effort => &mut self.seen_effort,
            RootField::ServiceTier => &mut self.seen_service_tier,
            RootField::EnvelopeType => &mut self.seen_envelope_type,
            RootField::Input => &mut self.seen_input,
            RootField::Tools => &mut self.seen_tools,
            RootField::Unknown => return Ok(()),
        };
        if *seen {
            return Err(ScanError);
        }
        *seen = true;
        Ok(())
    }

    pub(super) fn set_model(&mut self, value: DecodedString) -> Result<(), ScanError> {
        self.model = Some(identifier(value, true)?);
        Ok(())
    }

    pub(super) fn set_effort(&mut self, mut value: DecodedString) -> Result<(), ScanError> {
        // Capture only the closed wire vocabulary, never arbitrary response/user text.
        let effort = match value.as_str()? {
            "none" => "none",
            "minimal" => "minimal",
            "low" => "low",
            "medium" => "medium",
            "high" => "high",
            "xhigh" => "xhigh",
            "max" => "max",
            "ultra" => "ultra",
            _ => return Err(ScanError),
        };
        self.reasoning_effort = Some(effort.into());
        value.erase();
        Ok(())
    }

    pub(super) fn set_service_tier(
        &mut self,
        value: Option<DecodedString>,
    ) -> Result<(), ScanError> {
        self.service_tier = value.map(|item| identifier(item, false)).transpose()?;
        Ok(())
    }

    pub(super) fn set_envelope_type(
        &mut self,
        value: Option<DecodedString>,
    ) -> Result<(), ScanError> {
        self.envelope_type = value.map(|item| identifier(item, false)).transpose()?;
        Ok(())
    }

    pub(super) fn set_input_count(&mut self, count: usize) {
        self.input_count = count;
    }

    pub(super) fn set_tool_count(&mut self, count: usize) {
        self.tool_count = count;
    }

    pub(super) fn finish(self) -> Result<ScannedProjection, ScanError> {
        Ok(ScannedProjection {
            model: self.model.ok_or(ScanError)?,
            reasoning_effort: self.reasoning_effort,
            service_tier: self.service_tier,
            envelope_type: self.envelope_type,
            input_count: self.input_count,
            tool_count: self.tool_count,
            forbidden_field_present: self.forbidden_field_present,
        })
    }
}

fn identifier(mut decoded: DecodedString, model: bool) -> Result<String, ScanError> {
    let value = decoded.as_str()?;
    let characters_valid = value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'));
    let model_valid = !model || crate::services::llm::runtime_models::valid_model_id(value);
    if value.is_empty() || !characters_valid || !model_valid {
        return Err(ScanError);
    }
    let safe = value.to_string();
    decoded.erase();
    Ok(safe)
}
