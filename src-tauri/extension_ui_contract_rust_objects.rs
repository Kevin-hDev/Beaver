use serde_json::Value;

pub fn render(output: &mut String, contract: &Value) -> Result<(), String> {
    render_placements(output, contract)?;
    render_protected_occupants(output, contract)?;
    render_validation(output, contract)
}

fn render_placements(output: &mut String, contract: &Value) -> Result<(), String> {
    output.push_str(
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub struct UiPlacement {\n\
         \x20   pub key: &'static str,\n\
         \x20   pub contribution_type: &'static str,\n\
         \x20   pub cardinality: &'static str,\n\
         \x20   pub scope: &'static str,\n\
         \x20   pub third_party_chat_allowed: Option<bool>,\n\
         }\n\
         #[allow(dead_code)]\n\
         pub const UI_PLACEMENTS: &[UiPlacement] = &[\n",
    );
    for placement in array(contract, "placements")? {
        let third_party = match placement.get("thirdPartyChatAllowed") {
            Some(value) => format!(
                "Some({})",
                value
                    .as_bool()
                    .ok_or_else(|| "invalid UI placement chat policy".to_string())?
            ),
            None => "None".to_string(),
        };
        output.push_str(&format!(
            "    UiPlacement {{ key: {:?}, contribution_type: {:?}, cardinality: {:?}, scope: {:?}, third_party_chat_allowed: {third_party} }},\n",
            string(placement, "key")?,
            string(placement, "contributionType")?,
            string(placement, "cardinality")?,
            string(placement, "scope")?,
        ));
    }
    output.push_str("];\n");
    Ok(())
}

fn render_protected_occupants(output: &mut String, contract: &Value) -> Result<(), String> {
    output.push_str(
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub struct UiProtectedOccupant {\n\
         \x20   pub placement: &'static str,\n\
         \x20   pub occupant: &'static str,\n\
         \x20   pub operations: &'static [&'static str],\n\
         }\n\
         #[allow(dead_code)]\n\
         pub const UI_PROTECTED_OCCUPANTS: &[UiProtectedOccupant] = &[\n",
    );
    for occupant in array(contract, "protectedOccupants")? {
        let operations = array(occupant, "operations")?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(|value| format!("{value:?}"))
                    .ok_or_else(|| "invalid protected UI operation".to_string())
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");
        output.push_str(&format!(
            "    UiProtectedOccupant {{ placement: {:?}, occupant: {:?}, operations: &[{operations}] }},\n",
            string(occupant, "placement")?,
            string(occupant, "occupant")?,
        ));
    }
    output.push_str("];\n");
    Ok(())
}

fn render_validation(output: &mut String, contract: &Value) -> Result<(), String> {
    let validation = &contract["validation"];
    output.push_str(
        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n\
         pub struct UiValidation {\n\
         \x20   pub max_numeric_limit: usize,\n\
         \x20   pub min_order: i64,\n\
         \x20   pub max_order: i64,\n\
         \x20   pub theme_value_pattern: &'static str,\n\
         }\n",
    );
    output.push_str(&format!(
        "#[allow(dead_code)]\npub const UI_VALIDATION: UiValidation = UiValidation {{ max_numeric_limit: {}, min_order: {}, max_order: {}, theme_value_pattern: {:?} }};\n",
        unsigned(validation, "maxNumericLimit")?,
        signed(validation, "minOrder")?,
        signed(validation, "maxOrder")?,
        string(validation, "themeValuePattern")?,
    ));
    Ok(())
}

fn array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value], String> {
    value[key]
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| format!("missing {key}"))
}

fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value[key].as_str().ok_or_else(|| format!("missing {key}"))
}

fn unsigned(value: &Value, key: &str) -> Result<u64, String> {
    value[key].as_u64().ok_or_else(|| format!("missing {key}"))
}

fn signed(value: &Value, key: &str) -> Result<i64, String> {
    value[key].as_i64().ok_or_else(|| format!("missing {key}"))
}
