use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
#[serde(default)]
pub(super) struct ProfileDocumentV1 {
    pub schema_version: u16,
    pub automatic_enabled: bool,
    pub global_profile_id: String,
    pub global_selection_revision: u64,
    #[serde(deserialize_with = "bounded_profiles")]
    pub profiles: Vec<ProfileV1>,
}

impl Default for ProfileDocumentV1 {
    fn default() -> Self {
        Self {
            schema_version: 1,
            automatic_enabled: true,
            global_profile_id: super::profile_defaults::BEAVER_PROFILE_ID.into(),
            global_selection_revision: 1,
            profiles: Vec::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub(super) struct ProfileV1 {
    pub id: String,
    pub name: String,
    pub revision: u64,
    pub threshold_percent: u8,
    pub allow_under_64k: bool,
    pub summary: SummaryV1,
    pub system_prompt: Option<String>,
    pub handoff_prompt: Option<String>,
}

impl Default for ProfileV1 {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            revision: 1,
            threshold_percent: 90,
            allow_under_64k: false,
            summary: SummaryV1::default(),
            system_prompt: None,
            handoff_prompt: None,
        }
    }
}

impl ProfileV1 {
    pub fn system_prompt(&self) -> Option<&str> {
        self.summary
            .system_prompt
            .as_deref()
            .or(self.system_prompt.as_deref())
    }

    pub fn handoff_prompt(&self) -> Option<&str> {
        self.summary
            .handoff_prompt
            .as_deref()
            .or(self.handoff_prompt.as_deref())
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub(super) struct SummaryV1 {
    pub system_prompt: Option<String>,
    pub handoff_prompt: Option<String>,
}

fn bounded_profiles<'de, D>(deserializer: D) -> Result<Vec<ProfileV1>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ProfilesVisitor;

    impl<'de> Visitor<'de> for ProfilesVisitor {
        type Value = Vec<ProfileV1>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a bounded compression profile list")
        }

        fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut profiles =
                Vec::with_capacity(super::profile_limits::MAX_PROFILE_READ_CANDIDATES);
            while let Some(profile) = sequence.next_element()? {
                if profiles.len() < super::profile_limits::MAX_PROFILE_READ_CANDIDATES {
                    profiles.push(profile);
                }
            }
            Ok(profiles)
        }
    }

    deserializer.deserialize_seq(ProfilesVisitor)
}
