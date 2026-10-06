use super::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    pub calls_per_day: u32,
    pub calls_per_window: u32,
    pub tools_per_window: u32,
    pub actions_per_window: u32,
    pub max_input_bytes: usize,
    pub output_tokens: u32,
    pub micro_usd_per_day: u64,
    pub memory_bytes: u64,
}
impl Default for Budget {
    fn default() -> Self {
        Self {
            calls_per_day: 96,
            calls_per_window: 8,
            tools_per_window: 32,
            actions_per_window: 10,
            max_input_bytes: 64_000,
            output_tokens: 2048,
            micro_usd_per_day: 1_000_000,
            memory_bytes: 16_777_216,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub kind: String,
    pub model: String,
    pub url: Option<String>,
    pub api_key_env: Option<String>,
    /// Pinned Ollama digest; remote model identity is recorded on every response.
    pub digest: Option<String>,
    pub context: u32,
    pub timeout_seconds: u64,
    pub reserve_micro_usd: u64,
    pub resident_bytes: Option<u64>,
    pub provider: Option<String>,
    #[serde(default)]
    pub cpu_only: bool,
    #[serde(default)]
    pub thinking: bool,
}
impl Model {
    pub fn validate(&self) -> Result<()> {
        if self.model.is_empty() || self.timeout_seconds == 0 || self.context < 4096 {
            return Err("Modell, Kontext >=4096 und positive Frist erforderlich".into());
        }
        if !["mock", "ollama", "openrouter"].contains(&self.kind.as_str()) {
            return Err("Unbekannte Modellart".into());
        }
        if self.kind == "mock" {
            return Ok(());
        }
        let url = reqwest::Url::parse(self.url.as_deref().ok_or("Modell-URL fehlt")?)
            .map_err(|e| e.to_string())?;
        if !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err("Keine Geheimnisse oder Query in Modell-URL".into());
        }
        if self.kind == "ollama" {
            if !matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"))
                || url.scheme() != "http"
            {
                return Err("Ollama nur an explizitem lokalen HTTP-Endpunkt".into());
            }
        } else if url.as_str().trim_end_matches('/') != "https://openrouter.ai/api/v1"
            || self.api_key_env.as_ref().is_none_or(|s| s.is_empty())
            || self.reserve_micro_usd == 0
        {
            return Err("OpenRouter benötigt offiziellen HTTPS-Endpunkt, Schlüsselvariable und Kostenreservierung".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Role {
    pub id: String,
    pub purpose: String,
    pub model: String,
    pub capabilities: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Harness {
    pub revision: u64,
    pub roles: Vec<Role>,
}
impl Harness {
    pub fn validate(&self, models: &BTreeMap<String, Model>) -> Result<()> {
        if self.roles.is_empty() || self.roles.len() > 8 {
            return Err("Harness benötigt 1 bis 8 Rollen".into());
        }
        let all = kern::aktion::erlaubte_typen(kern::Rolle::Alle);
        let mut ids = BTreeSet::new();
        for role in &self.roles {
            if !identifier(&role.id)
                || !ids.insert(&role.id)
                || role.purpose.len() > 4000
                || !models.contains_key(&role.model)
                || role.capabilities.len() > all.len()
                || role.capabilities.iter().any(|c| !all.contains(&c.as_str()))
            {
                return Err("Ungültige Rolle, Modellzuordnung oder Fähigkeit".into());
            }
        }
        Ok(())
    }
}
pub fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub seed: u64,
    pub days: i64,
    pub mode: String,
    pub interval_hours: u32,
    pub budget: Budget,
    pub models: BTreeMap<String, Model>,
    pub players: Vec<Harness>,
    /// Host memory admission cap, not an inference quality knob.
    pub local_memory_bytes: u64,
    pub parallel: usize,
    /// Explicit isolation mode: trusted tools, Windows LPAC, or Docker workers.
    pub sandbox: String,
    pub sandbox_image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sandbox_worker: Option<String>,
    pub learning_package: Option<String>,
    pub learning_sha256: Option<String>,
    #[serde(default)]
    pub allow_model_switch: bool,
    /// Empty only for the v2 compatibility contract.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub provider_mode: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub primary_models: BTreeMap<u16, String>,
    /// One strategist uses internal roles as an editable organization, without a call per role.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub coordinated_roles: bool,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub window_seconds: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_epoch: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub deadline_policy: String,
    /// Bounded catch-up slices of the same frozen world; never grants another player budget.
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub max_window_slices: u32,
}
fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}
fn is_zero(v: &u64) -> bool {
    *v == 0
}
impl Config {
    pub fn protocol(&self) -> &'static str {
        match self.version {
            4 => "persistent-players-v4",
            3 => "persistent-players-v3",
            _ => "persistent-players-v2",
        }
    }
    /// Rule code changes matter even when the numeric rulebook is byte-identical.
    pub fn rules_contract(&self) -> serde_json::Value {
        serde_json::json!({"labor_version":self.version,
            "rules_sha256":crate::journal::hash(crate::RULES.as_bytes())})
    }
    pub fn v4(players: usize) -> Self {
        let mut c = Self::v3(players);
        c.version = 4;
        c.deadline_policy = "controlled".into();
        c.max_window_slices = 4;
        for h in &mut c.players {
            for r in &mut h.roles {
                r.capabilities.push("flotte_versorgen".into());
            }
        }
        c
    }
    pub fn execution_roles(&self, sid: u16, h: &Harness) -> Vec<Role> {
        if !self.coordinated_roles {
            return h.roles.clone();
        }
        vec![Role { id: "koordination".into(),
            purpose: "Du bist der strategische Hauptagent. Organisiere die Rollen aus deinem Lagebild selbst über Aufgaben, Pläne und harness_patch. Mit tool_select lädst du benötigte Werkzeuge gezielt für den nächsten Aufruf.".into(),
            model: self.primary_models.get(&sid).cloned().unwrap_or_else(|| h.roles[0].model.clone()),
            capabilities: h.roles.iter().flat_map(|r|r.capabilities.clone()).collect::<BTreeSet<_>>().into_iter().collect() }]
    }
    pub fn v3(players: usize) -> Self {
        let mut c = Self::demo(players);
        c.version = 3;
        c.provider_mode = "mock".into();
        c.coordinated_roles = true;
        c.window_seconds = 900;
        c.interval_hours = 1;
        c.budget.calls_per_day = 768;
        c
    }
    pub fn demo(players: usize) -> Self {
        let model = Model {
            kind: "mock".into(),
            model: "labor-demo-v2".into(),
            url: None,
            api_key_env: None,
            digest: None,
            context: 32768,
            timeout_seconds: 60,
            reserve_micro_usd: 0,
            resident_bytes: None,
            provider: None,
            cpu_only: false,
            thinking: false,
        };
        let role = Role {
            id: "regierung".into(),
            purpose: "Versorgung sichern, Wirtschaft aufbauen, Nachbarn und Expansion planen."
                .into(),
            model: "demo".into(),
            capabilities: kern::aktion::erlaubte_typen(kern::Rolle::Alle)
                .into_iter()
                .filter(|name| *name != "flotte_versorgen")
                .map(str::to_string)
                .collect(),
        };
        Self {
            version: 2,
            seed: 42,
            days: 365,
            mode: "cold_start".into(),
            interval_hours: 4,
            budget: Budget::default(),
            models: BTreeMap::from([("demo".into(), model)]),
            players: vec![
                Harness {
                    revision: 0,
                    roles: vec![role]
                };
                players
            ],
            local_memory_bytes: 0,
            parallel: 1,
            sandbox: "trusted".into(),
            sandbox_image: None,
            sandbox_worker: None,
            learning_package: None,
            learning_sha256: None,
            allow_model_switch: false,
            provider_mode: String::new(),
            primary_models: BTreeMap::new(),
            coordinated_roles: false,
            window_seconds: 0,
            previous_epoch: None,
            deadline_policy: String::new(),
            max_window_slices: 0,
        }
    }
    pub fn validate(&self) -> Result<()> {
        if ![2, 3, 4].contains(&self.version)
            || !(2..=50).contains(&self.players.len())
            || !(1..=365).contains(&self.days)
            || !(1..=24).contains(&self.interval_hours)
            || !(1..=32).contains(&self.parallel)
            || !["cold_start", "shared_learning", "continuation"].contains(&self.mode.as_str())
        {
            return Err("Laufvertrag: Version 2/3/4, 2..50 Spieler, 1..365 Tage, 1..24h Takt, gültiger Modus erforderlich".into());
        }
        if self.version >= 3 {
            if !["mock", "local_only", "mixed", "local_or_remote"]
                .contains(&self.provider_mode.as_str())
                || !(1..=900).contains(&self.window_seconds)
            {
                return Err(
                    "Version 3 braucht expliziten Providermodus und 1..900 Sekunden Fensterfrist"
                        .into(),
                );
            }
            for m in self.models.values() {
                if (self.provider_mode == "local_only" && m.kind != "ollama")
                    || (self.provider_mode == "mock" && m.kind != "mock")
                    || (self.provider_mode != "mock" && m.kind == "mock")
                {
                    return Err(
                        "Modellpool verletzt Providermodus; kein stiller Providerwechsel".into(),
                    );
                }
            }
        } else if !self.provider_mode.is_empty()
            || self.coordinated_roles
            || self.window_seconds != 0
            || !self.primary_models.is_empty()
        {
            return Err("Neue Schedulerfelder benötigen Version 3".into());
        }
        if (self.version >= 4
            && !["controlled", "throughput"].contains(&self.deadline_policy.as_str()))
            || (self.version < 4 && !self.deadline_policy.is_empty())
        {
            return Err("Version 4 benötigt deadline_policy controlled oder throughput; ältere Verträge bleiben unverändert".into());
        }
        if (self.version >= 4 && !(1..=16).contains(&self.max_window_slices))
            || (self.version < 4 && self.max_window_slices != 0)
        {
            return Err(
                "Version 4 benötigt 1..16 begrenzte Arbeitsabschnitte je eingefrorenem Fenster"
                    .into(),
            );
        }
        if self
            .primary_models
            .iter()
            .any(|(sid, m)| *sid as usize >= self.players.len() || !self.models.contains_key(m))
        {
            return Err("Ungültiges Spieler-Hauptmodell".into());
        }
        let b = &self.budget;
        if b.calls_per_day == 0
            || b.calls_per_window == 0
            || b.calls_per_window > b.calls_per_day
            || b.tools_per_window == 0
            || !(1..=32).contains(&b.actions_per_window)
            || b.max_input_bytes < 4096
            || b.max_input_bytes > 1_000_000
            || b.output_tokens == 0
            || !(65536..=16_777_216).contains(&b.memory_bytes)
        {
            return Err("Ungültiges gemeinsames Spielerbudget".into());
        }
        if !["trusted", "docker", "native"].contains(&self.sandbox.as_str())
            || (self.sandbox == "docker" && self.sandbox_image.is_none())
        {
            return Err("Sandbox-Modus oder Image fehlt".into());
        }
        if self.models.is_empty() {
            return Err("Modellpool leer".into());
        }
        for (name, m) in &self.models {
            if !identifier(name) {
                return Err("Ungültige Modell-ID".into());
            }
            m.validate()?;
        }
        for h in &self.players {
            h.validate(&self.models)?;
        }
        if (self.mode == "continuation") != self.previous_epoch.is_some()
            || (self.previous_epoch.is_some() && self.version < 3)
        {
            return Err("Fortsetzung braucht Version 3 und previous_epoch; getrennt von Nullstart und gemeinsamem Lernpaket".into());
        }
        if self.mode == "shared_learning" {
            if self.learning_package.is_none()
                || self.learning_sha256.as_ref().is_none_or(|s| s.len() != 64)
            {
                return Err("Gemeinsames Lernpaket und SHA-256 erforderlich".into());
            }
        } else if self.learning_package.is_some() || self.learning_sha256.is_some() {
            return Err("Nullstart hat kein Lernpaket".into());
        }
        Ok(())
    }
}
