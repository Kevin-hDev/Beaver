use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionWindowBand {
    #[serde(rename = "under_64k")]
    Under64K,
    Compact,
    Large,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetMode {
    Fixed,
    Percentage,
    Minimum,
}

// Task 11 consumes this type when the compression entry points share one orchestrator.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionTrigger {
    Automatic,
    Explicit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompressionCategory {
    UserMessages,
    AssistantMessages,
    EvidenceEnvelope,
    Tools,
    Files,
    TextAttachments,
    Images,
    Git,
    PlanAndTasks,
    Subagents,
    UnresolvedState,
    CriticalReferences,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenBudget {
    pub mode: BudgetMode,
    pub fixed_tokens: u32,
    pub percent_basis_points: u16,
    /// Lower bound used by Beaver's clamped factory policies. User-selected
    /// fixed, percentage and minimum modes keep this at zero.
    #[serde(default)]
    pub minimum_tokens: u32,
}

impl TokenBudget {
    pub const fn fixed(tokens: u32) -> Self {
        Self {
            mode: BudgetMode::Fixed,
            fixed_tokens: tokens,
            percent_basis_points: 0,
            minimum_tokens: 0,
        }
    }

    pub const fn percentage(percent_basis_points: u16) -> Self {
        Self {
            mode: BudgetMode::Percentage,
            fixed_tokens: 0,
            percent_basis_points,
            minimum_tokens: 0,
        }
    }

    pub const fn minimum(fixed_tokens: u32, percent_basis_points: u16) -> Self {
        Self {
            mode: BudgetMode::Minimum,
            fixed_tokens,
            percent_basis_points,
            minimum_tokens: 0,
        }
    }

    pub const fn clamped(
        percent_basis_points: u16,
        minimum_tokens: u32,
        maximum_tokens: u32,
    ) -> Self {
        Self {
            mode: BudgetMode::Percentage,
            fixed_tokens: maximum_tokens,
            percent_basis_points,
            minimum_tokens,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CategoryBudget {
    pub enabled: bool,
    pub tokens: TokenBudget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemBudget {
    pub enabled: bool,
    pub max_items: u16,
    pub tokens_per_item: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageBudget {
    pub enabled: bool,
    pub max_items: u16,
    pub max_total_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryOutputBudget {
    pub window_limit: TokenBudget,
    pub input_ratio_divisor: u16,
    pub input_floor_tokens: u32,
    pub input_ceiling_tokens: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SummaryModelSelection {
    Current,
    Explicit { provider: String, model: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressionSummarySettings {
    pub enabled: bool,
    pub system_prompt: String,
    pub handoff_prompt: String,
    pub model: SummaryModelSelection,
    pub fallback_model: Option<SummaryModelSelection>,
    pub ordinary_retries: u8,
    pub input_budget: TokenBudget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressionBandSettings {
    pub target_percent: u8,
    pub response_reserve: TokenBudget,
    pub minimum_reduction: TokenBudget,
    pub summary_output: SummaryOutputBudget,
    pub user_messages: CategoryBudget,
    pub assistant_messages: CategoryBudget,
    pub evidence_envelope: TokenBudget,
    pub tools: ItemBudget,
    pub files: ItemBudget,
    pub text_attachments: ItemBudget,
    pub images: ImageBudget,
    pub git_tokens: u32,
    pub plan_and_tasks_tokens: u32,
    pub subagent_detail_tokens: u32,
    pub unresolved_state_tokens: u32,
    pub critical_references: ItemBudget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompressionProfile {
    pub id: String,
    pub name: String,
    pub revision: u64,
    pub threshold_percent: u8,
    pub allow_under_64k: bool,
    pub summary: CompressionSummarySettings,
    pub under_64k: CompressionBandSettings,
    pub compact: CompressionBandSettings,
    pub large: CompressionBandSettings,
    pub reduction_order: Vec<CompressionCategory>,
}
