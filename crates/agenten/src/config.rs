use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const ROLLEN: [&str; 4] = ["stratege", "verwalter", "feldherr", "diplomat"];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provider {
    pub art: String,
    pub modell: String,
    pub url: Option<String>,
    pub api_key_env: Option<String>,
    pub remote_erlaubt: bool,
    pub max_tokens: u32,
    pub timeout_sekunden: u64,
    /// Denken: "aus", "niedrig", "mittel" oder "hoch"; ohne Angabe wird nichts gesendet.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub denken: Option<String>,
    /// Antwortschema der Engine erzwingen (response_format json_schema, strict) statt nur json_object.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub schema: bool,
    /// Zusätzliche OpenRouter-Providervorgaben, etwa order oder data_collection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<Value>,
    /// Versuche je Aufruf bei eindeutigem Fehler (HTTP-Status, leere oder abgeschnittene Antwort).
    /// Neue Felder stehen nur bei abweichendem Wert im Journal, damit ältere Läufe fortsetzbar bleiben.
    #[serde(default = "zwei_versuche", skip_serializing_if = "ist_zwei")]
    pub versuche: u32,
}

fn zwei_versuche() -> u32 {
    2
}

fn ist_zwei(n: &u32) -> bool {
    *n == 2
}

pub const DENKEN: [&str; 4] = ["aus", "niedrig", "mittel", "hoch"];

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub startwert: u64,
    pub spieler: usize,
    pub tage: i64,
    pub parallel: usize,
    pub max_anfragen: usize,
    pub anbieter: BTreeMap<String, Provider>,
    pub rollen: BTreeMap<String, String>,
}

impl Config {
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=50).contains(&self.spieler)
            || !(1..=365).contains(&self.tage)
            || !(1..=200).contains(&self.parallel)
            || self.max_anfragen == 0
        {
            return Err(
                "spieler 1..50, tage 1..365, parallel 1..200 und max_anfragen > 0 erforderlich"
                    .into(),
            );
        }
        if self.rollen.len() != 4 || !ROLLEN.iter().all(|r| self.rollen.contains_key(*r)) {
            return Err("Genau stratege, verwalter, feldherr, diplomat konfigurieren".into());
        }
        for name in self.rollen.values() {
            let p = self
                .anbieter
                .get(name)
                .ok_or_else(|| format!("Anbieter {name} fehlt"))?;
            p.validate()?;
        }
        Ok(())
    }
    pub fn demo(spieler: usize) -> Self {
        Self {
            startwert: 42,
            spieler,
            tage: 1,
            parallel: 8,
            max_anfragen: 10_000,
            anbieter: BTreeMap::from([(
                "offline".into(),
                Provider {
                    art: "mock".into(),
                    modell: "regel-demo-v1".into(),
                    url: None,
                    api_key_env: None,
                    remote_erlaubt: false,
                    max_tokens: 1024,
                    timeout_sekunden: 30,
                    denken: None,
                    schema: false,
                    provider: None,
                    versuche: 2,
                },
            )]),
            rollen: ROLLEN
                .iter()
                .map(|r| (r.to_string(), "offline".into()))
                .collect(),
        }
    }
}

impl Provider {
    pub fn validate(&self) -> Result<(), String> {
        if self.modell.trim().is_empty() || self.max_tokens == 0 || self.timeout_sekunden == 0 {
            return Err("Modell, positive Token- und Zeitgrenze erforderlich".into());
        }
        if !(1..=5).contains(&self.versuche) {
            return Err("versuche muss 1 bis 5 sein".into());
        }
        if self.denken.as_deref().is_some_and(|d| !DENKEN.contains(&d)) {
            return Err(format!("denken muss {} sein", DENKEN.join(", ")));
        }
        if self.provider.as_ref().is_some_and(|p| !p.is_object()) {
            return Err("provider muss ein JSON-Objekt sein".into());
        }
        if self.art == "mock" {
            return Ok(());
        }
        if !["local", "openrouter"].contains(&self.art.as_str()) {
            return Err("Unbekannter Anbieter".into());
        }
        let url = reqwest::Url::parse(self.url.as_deref().ok_or("URL fehlt")?)
            .map_err(|_| "URL ungültig")?;
        if !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err("URL darf keine Zugangsdaten, Query oder Fragment enthalten".into());
        }
        let local = matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"));
        if !["http", "https"].contains(&url.scheme())
            || (!local && (!self.remote_erlaubt || url.scheme() != "https"))
        {
            return Err("Externe Modellaufrufe benötigen HTTPS und remote_erlaubt".into());
        }
        if self.art == "openrouter"
            && (url.as_str().trim_end_matches('/') != "https://openrouter.ai/api/v1"
                || self.api_key_env.as_deref().unwrap_or("").is_empty())
        {
            return Err("OpenRouter benötigt offiziellen HTTPS-Endpunkt und api_key_env".into());
        }
        Ok(())
    }
}
