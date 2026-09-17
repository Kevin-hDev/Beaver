use super::{PromptMatrix, PromptPair, SystemPromptSettings, MAX_MODELS};
use crate::services::agent_local::system_prompt_types::{PromptMode, PromptTier};

impl SystemPromptSettings {
    pub fn select_ollama_beaver(
        &mut self,
        model: &str,
        mode: PromptMode,
        tier: PromptTier,
    ) -> Result<(), String> {
        super::super::model_customizations::validate_model_name(model)?;
        self.ensure_model_capacity(model)?;
        let matrix = self.ollama.entry(model.to_string()).or_default();
        *matrix.get_mut(mode, tier) = None;
        matrix.set_beaver(mode, tier, true);
        Ok(())
    }

    pub fn restore_ollama_default(
        &mut self,
        model: &str,
        mode: PromptMode,
        tier: PromptTier,
    ) -> Result<(), String> {
        super::super::model_customizations::validate_model_name(model)?;
        let Some(matrix) = self.ollama.get_mut(model) else {
            return Ok(());
        };
        *matrix.get_mut(mode, tier) = None;
        matrix.set_beaver(mode, tier, false);
        if matrix.is_empty() {
            self.ollama.remove(model);
        }
        Ok(())
    }

    pub fn ollama_uses_beaver(
        &self,
        model: &str,
        mode: PromptMode,
        tier: PromptTier,
    ) -> bool {
        self.ollama
            .get(model)
            .is_some_and(|matrix| matrix.beaver(mode, tier))
    }

    pub(super) fn ensure_model_capacity(&self, model: &str) -> Result<(), String> {
        if self.ollama.len() >= MAX_MODELS && !self.ollama.contains_key(model) {
            return Err("system-prompt-model-limit".into());
        }
        Ok(())
    }
}

impl PromptMatrix {
    pub(super) fn beaver(&self, mode: PromptMode, tier: PromptTier) -> bool {
        self.pair(mode).beaver(tier)
    }

    pub(super) fn set_beaver(&mut self, mode: PromptMode, tier: PromptTier, enabled: bool) {
        self.pair_mut(mode).set_beaver(tier, enabled);
    }
}

impl PromptPair {
    fn beaver(&self, tier: PromptTier) -> bool {
        match tier {
            PromptTier::Compact => self.compact_beaver,
            PromptTier::Detailed => self.detailed_beaver,
        }
    }

    fn set_beaver(&mut self, tier: PromptTier, enabled: bool) {
        match tier {
            PromptTier::Compact => self.compact_beaver = enabled,
            PromptTier::Detailed => self.detailed_beaver = enabled,
        }
    }
}
