use kern::{Einheit, Forschung, Gebaeude, Gut, Volk};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Explicit temporary reuse of existing art; never claim a new image was generated.
pub fn visual_key<'a>(category: &str, key: &'a str) -> &'a str {
    match (category,key) {
        ("buildings","geheimdienst")=>"sensorphalanx",
        ("research","ueberwachungstechnik")=>"spionagetechnik",
        ("research","abschirmtechnik")=>"schildtechnik",
        ("missions","system_erkunden"|"flotten_spionage")=>"spionage",
        ("missions","saven")=>"transport",
        _=>key,
    }
}
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Asset {
    pub id: String,
    pub category: String,
    pub key: String,
    pub faction: Option<String>,
    pub label: String,
    pub file: String,
    pub width: u32,
    pub height: u32,
    pub seed: u64,
    pub prompt: String,
    pub negative_prompt: String,
    #[serde(default)]
    pub gate: Value,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Catalog {
    pub version: u32,
    pub style: String,
    pub style_status: String,
    pub source_rule_sha256: String,
    pub headless_requires_assets: bool,
    pub factions: BTreeMap<String, Value>,
    pub assets: Vec<Asset>,
    pub counts: BTreeMap<String, usize>,
}
impl Catalog {
    pub fn load(content: impl AsRef<Path>) -> Result<Self, String> {
        read_json(&content.as_ref().join("catalog.json"))
    }
    pub fn asset(&self, id: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.id == id)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.headless_requires_assets {
            return Err("Headless must not require graphics".into());
        }
        let mut ids = BTreeSet::new();
        let mut paths = BTreeSet::new();
        let mut counts = BTreeMap::new();
        for a in &self.assets {
            if a.id.is_empty()
                || !a
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                || !ids.insert(&a.id)
            {
                return Err(format!("Invalid/duplicate asset id: {}", a.id));
            }
            safe_relative(&a.file)?;
            if !paths.insert(&a.file) {
                return Err(format!("Duplicate output path: {}", a.file));
            }
            if a.width < 64
                || a.height < 64
                || a.width > 4096
                || a.height > 4096
                || (a.gate["generator"] != "image_gen" && (a.width % 64 != 0 || a.height % 64 != 0))
            {
                return Err(format!("Invalid dimensions: {}", a.id));
            }
            if a.prompt.trim().is_empty() {
                return Err(format!("Missing prompt: {}", a.id));
            }
            *counts.entry(a.category.clone()).or_insert(0) += 1;
        }
        if counts != self.counts {
            return Err("Catalog category counts do not match assets".into());
        }
        let covered = |cat: &str, key: &str, faction: Option<&str>| {
            self.assets
                .iter()
                .any(|a| a.category == cat && a.key == key && a.faction.as_deref() == faction)
        };
        let check = |cat: &str, key: &str, faction: Option<&str>| {
            if covered(cat, key, faction) {
                Ok(())
            } else {
                Err(format!("Missing engine content: {cat}/{key}/{faction:?}"))
            }
        };
        for g in Gut::ALLE {
            check("resources", g.name(), None)?;
        }
        for f in Forschung::ALLE {
            check("research", visual_key("research",f.name()), None)?;
        }
        for b in Gebaeude::ALLE {
            check("buildings", visual_key("buildings",b.name()), None)?;
        }
        for v in Volk::ALLE {
            if !self.factions.contains_key(v.name()) {
                return Err(format!("Missing faction: {v}"));
            }
            check("portraits", v.name(), Some(v.name()))?;
            for b in Gebaeude::ALLE {
                check("buildings", visual_key("buildings",b.name()), Some(v.name()))?;
            }
            for e in Einheit::ALLE {
                check(
                    if e.ist_schiff() { "ships" } else { "defenses" },
                    e.name(),
                    Some(v.name()),
                )?;
            }
        }
        Ok(())
    }
    /// Rules may evolve independently. Callers display this as stale provenance, not silently refresh it.
    pub fn rules_current(&self, rules: impl AsRef<Path>) -> Result<bool, String> {
        Ok(self.source_rule_sha256 == digest(&fs::read(rules).map_err(|e| e.to_string())?))
    }
}
pub fn digest(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}
pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(text.trim_start_matches('\u{feff}'))
        .map_err(|e| format!("{}: {e}", path.display()))
}
pub fn safe_relative(name: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(name);
    if name.is_empty()
        || name.contains(':')
        || name.contains('\\')
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(format!("Unsafe content-relative path: {name}"));
    }
    Ok(path)
}

/// Reconcile authored new building motifs and rule gates using Rust engine types.
/// Existing prompts, reviewed files and all unrelated content metadata are preserved.
pub fn sync_engine(content: &Path) -> Result<usize, String> {
    let rules_path = content.join("../regeln/regelwerk.ron");
    let rules_text = fs::read_to_string(&rules_path).map_err(|e| e.to_string())?;
    let rules = kern::Regelwerk::laden(&rules_text)?;
    let mut document: Value = read_json(&content.join("catalog.json"))?;
    let original = document["assets"]
        .as_array()
        .ok_or("Missing assets")?
        .clone();
    let mut additions = Vec::new();
    for (key,label,subject) in [
        ("raketensilo","Raketensilo","a fortified underground planetary missile complex with blast doors, radar arrays and recessed launch tubes"),
        ("orbitalring","Orbitalring","a vast orbital ring circling a blue planet with spaced transport towers, ship docks and visible construction segments"),
        ("forschungsarchiv","Forschungsarchiv","a monumental scientific archive of crystalline data vaults, research laboratories and precision sensor arrays"),
        ("versorgungsnetz","Versorgungsnetz","a planetary logistics network hub with automated cargo conveyors, storage reservoirs and coordinated freight terminals"),
    ] {
        let Some(building)=Gebaeude::aus_name(key) else {continue;};
        if !rules.gebaeude.contains_key(&building) {return Err(format!("Missing rule for {key}"));}
        for faction in std::iter::once(None).chain(Volk::ALLE.into_iter().map(|v|Some(v.name()))) {
            if original.iter().any(|a|a["category"]=="buildings" && a["key"]==key && a["faction"].as_str()==faction){continue;}
            let mut a=original.iter().find(|a|a["category"]=="buildings"&&a["key"]=="orbitalwerft"&&a["faction"].as_str()==faction).ok_or("Missing visual style template")?.clone();
            let id=format!("buildings.{key}{}",faction.map(|f|format!(".{f}")).unwrap_or_default());
            let prompt=a["prompt"].as_str().ok_or("Missing template prompt")?;
            let prefix=prompt.split("Subject:").next().ok_or("Missing template subject")?;
            a["prompt"]=serde_json::json!(format!("{prefix}Subject: {subject}."));
            a["id"]=serde_json::json!(id);a["key"]=serde_json::json!(key);a["label"]=serde_json::json!(label);
            a["file"]=serde_json::json!(format!("assets/buildings/{id}.png"));
            let hash=Sha256::digest(id.as_bytes());a["seed"]=serde_json::json!(u32::from_le_bytes(hash[..4].try_into().unwrap()));
            a["provenance"]=serde_json::json!("planned_qwen_image_2.1");a["delivery"]["review_status"]=serde_json::json!("not_generated");
            additions.push(a);
        }
    }
    let added = additions.len();
    let assets = document["assets"].as_array_mut().unwrap();
    assets.extend(additions);
    for a in assets.iter_mut() {
        let key = a["key"].as_str().unwrap_or("");
        let gate=match a["category"].as_str().unwrap_or("") {
            "buildings"=>Gebaeude::aus_name(key).and_then(|g|rules.gebaeude.get(&g)).map(|r|serde_json::json!({"ab_stufe":r.ab_stufe,"braucht":r.braucht,"wirkung":r.wirkung})),
            "research"=>Forschung::aus_name(key).and_then(|f|rules.forschung.get(&f)).map(|r|serde_json::json!({"ab_stufe":r.ab_stufe,"labor":r.labor,"wirkung":r.wirkung})),
            "ships"|"defenses"=>Einheit::aus_name(key).and_then(|e|rules.einheiten.get(&e)).map(|r|serde_json::json!({"ab_stufe":r.ab_stufe,"werft":r.werft,"braucht":r.braucht})),_=>None,
        };
        if let Some(g) = gate {
            a["gate"] = g;
        }
    }
    let mut counts = BTreeMap::<String, usize>::new();
    for a in assets.iter() {
        *counts
            .entry(a["category"].as_str().ok_or("Missing category")?.into())
            .or_default() += 1;
    }
    document["counts"] = serde_json::to_value(counts).map_err(|e| e.to_string())?;
    document["source_rule_sha256"] = serde_json::json!(digest(rules_text.as_bytes()));
    let typed: Catalog = serde_json::from_value(document.clone()).map_err(|e| e.to_string())?;
    typed.validate()?;
    crate::production::save_json(&content.join("catalog.json"), &document)?;
    fs::write(
        content.join("catalog.js"),
        format!(
            "window.STERNEN_CONTENT = {};\n",
            serde_json::to_string(&document).map_err(|e| e.to_string())?
        ),
    )
    .map_err(|e| e.to_string())?;
    Ok(added)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn checked_in_catalog_covers_engine() {
        let c = Catalog::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content")).unwrap();
        c.validate().unwrap();
    }
    #[test]
    fn detects_missing_engine_content() {
        let mut c =
            Catalog::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content")).unwrap();
        c.assets
            .retain(|a| !(a.category == "ships" && a.key == "kleiner_transporter"));
        c.counts.insert(
            "ships".into(),
            c.assets.iter().filter(|a| a.category == "ships").count(),
        );
        assert!(c.validate().unwrap_err().contains("Missing engine content"));
    }
    #[test]
    fn rejects_path_escape() {
        for s in ["../secret", "/root", "C:/secret", "..\\secret", ""] {
            assert!(safe_relative(s).is_err());
        }
    }
}
