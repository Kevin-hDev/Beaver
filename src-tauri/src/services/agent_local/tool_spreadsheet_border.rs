use serde_json::Value;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct BorderSides {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

impl BorderSides {
    pub(super) const fn all() -> Self {
        Self {
            top: true,
            bottom: true,
            left: true,
            right: true,
        }
    }

    pub(super) const fn is_empty(self) -> bool {
        !self.top && !self.bottom && !self.left && !self.right
    }
}

pub(super) fn parse(value: &Value) -> Result<Option<BorderSides>, String> {
    if value.is_null() {
        return Ok(None);
    }
    let values = value
        .as_array()
        .ok_or_else(|| "border_sides doit être un tableau".to_string())?;
    if values.len() > 4 {
        return Err("border_sides accepte au maximum 4 côtés".into());
    }

    let mut sides = BorderSides::default();
    for value in values {
        let side = value
            .as_str()
            .ok_or_else(|| "Chaque côté de border_sides doit être un texte".to_string())?;
        if side.eq_ignore_ascii_case("top") {
            sides.top = true;
        } else if side.eq_ignore_ascii_case("bottom") {
            sides.bottom = true;
        } else if side.eq_ignore_ascii_case("left") {
            sides.left = true;
        } else if side.eq_ignore_ascii_case("right") {
            sides.right = true;
        } else {
            return Err(format!("Côté de bordure inconnu: {side}"));
        }
    }
    Ok(Some(sides))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_and_oversized_side_lists() {
        assert!(parse(&serde_json::json!(["diagonal"])).is_err());
        assert!(parse(&serde_json::json!(["top", "bottom", "left", "right", "top"])).is_err());
    }
}
