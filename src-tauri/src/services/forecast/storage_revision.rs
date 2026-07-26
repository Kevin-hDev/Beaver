pub fn next(stored: Option<u32>, incoming: u32) -> Result<u32, String> {
    match stored {
        None => Ok(incoming.max(1)),
        Some(stored) if stored == incoming => stored
            .checked_add(1)
            .ok_or_else(|| "Révision Forecast invalide".to_string()),
        Some(_) => Err("L'analyse Forecast a été modifiée".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_analysis_starts_at_one() {
        assert_eq!(next(None, 0), Ok(1));
    }

    #[test]
    fn current_revision_increments_once() {
        assert_eq!(next(Some(4), 4), Ok(5));
    }

    #[test]
    fn stale_or_overflowed_revision_fails_closed() {
        assert!(next(Some(4), 3).is_err());
        assert!(next(Some(u32::MAX), u32::MAX).is_err());
    }
}
