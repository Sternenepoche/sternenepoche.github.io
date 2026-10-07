//! Generated artwork is resolved by engine key and faction, then verified against provenance.
use crate::Player;
use eframe::egui;
use inhalt::catalog::{digest, safe_relative, Asset, Catalog};
use serde_json::Value;
use std::time::{Duration, Instant, SystemTime};
use super::Bildschirm;

type Stamp = Option<(SystemTime, u64)>;
pub(crate) struct ArtCacheEntry {
    texture: Option<egui::TextureHandle>,
    stamps: (Stamp, Stamp),
    checked: Instant,
}

fn stamp(path: &std::path::Path) -> Stamp {
    let meta = std::fs::metadata(path).ok()?;
    Some((meta.modified().ok()?, meta.len()))
}

/// Every catalog family has an in-game home, including variants and illustrations
/// which are not attached to a currently active order or event.
fn screen_categories(screen: Bildschirm) -> &'static [&'static str] {
    match screen {
        Bildschirm::Uebersicht => &["portraits", "emblems", "civilizations"],
        Bildschirm::Kolonie => &["surfaces", "planets", "stats", "resources", "materials"],
        Bildschirm::Gebaeude => &["buildings"],
        Bildschirm::Forschung => &["research"],
        Bildschirm::Werft => &["ships", "defenses", "ground_units", "missiles"],
        Bildschirm::Flotten => &["missions"],
        Bildschirm::Galaxie => &["celestials", "planets"],
        Bildschirm::Markt => &["resources"],
        Bildschirm::Diplomatie => &["diplomacy", "contracts"],
        Bildschirm::Regierung => &["progression", "civilizations", "role_portraits", "emblems"],
        Bildschirm::Berichte => &["events"],
        Bildschirm::Rangliste => &["emblems", "stats"],
        Bildschirm::Anleitung => &["backgrounds", "materials"],
        _ => &[],
    }
}

fn select_asset<'a>(catalog: &'a Catalog, category: &str, key: &str, faction: &str) -> Option<&'a Asset> {
    let key=inhalt::catalog::visual_key(category,key);
    catalog.assets.iter().find(|a| a.category == category && a.key == key && a.faction.as_deref() == Some(faction))
        .or_else(|| catalog.assets.iter().find(|a| a.category == category && a.key == key && a.faction.is_none()))
}

impl Player {
    pub(crate) fn motiv_groesse(&mut self, ui: &mut egui::Ui, category: &str, key: &str, view: &Value, size: egui::Vec2) -> bool {
        let faction = view["volk"].as_str().unwrap_or("").to_lowercase();
        let asset = self.catalog.as_ref().and_then(|c| select_asset(c, category, key, &faction)).cloned();
        asset.is_some_and(|a| self.motiv_anzeigen(ui, &a, size, false))
    }

    pub(crate) fn motiv_klein(&mut self, ui: &mut egui::Ui, category: &str, key: &str, view: &Value) {
        self.motiv_groesse(ui, category, key, view, egui::vec2(30.0, 26.0));
    }

    pub(crate) fn bildkopf(&mut self, ui: &mut egui::Ui, view: &Value) {
        let (background, role) = match self.screen {
            Bildschirm::Uebersicht => ("start", "verwalter"),
            Bildschirm::Kolonie | Bildschirm::Gebaeude => ("command", "verwalter"),
            Bildschirm::Forschung => ("research", "stratege"),
            Bildschirm::Werft => ("shipyard", "feldherr"),
            Bildschirm::Flotten => ("hangar", "feldherr"),
            Bildschirm::Galaxie => ("stars", "stratege"),
            Bildschirm::Markt => ("market", "verwalter"),
            Bildschirm::Diplomatie => ("diplomacy", "diplomat"),
            Bildschirm::Regierung => ("command", "stratege"),
            Bildschirm::Berichte => ("archive", ""),
            Bildschirm::Rangliste => ("ranking", ""),
            Bildschirm::Anleitung => ("nebula", ""),
            _ => return,
        };
        ui.horizontal(|ui| {
            self.motiv_groesse(ui, "backgrounds", background, view, egui::vec2(208.0, 78.0));
            if !role.is_empty() { self.motiv_groesse(ui, "role_portraits", role, view, egui::vec2(64.0, 78.0)); }
            let faction = view["volk"].as_str().unwrap_or("");
            self.motiv_groesse(ui, "emblems", faction, view, egui::vec2(58.0, 58.0));
        });
    }

    pub(crate) fn bildsammlung(&mut self, ui: &mut egui::Ui, view: &Value) {
        let categories = screen_categories(self.screen);
        if categories.is_empty() { return; }
        let faction = view["volk"].as_str().unwrap_or("");
        let assets: Vec<_> = self.catalog.as_ref().into_iter().flat_map(|c| &c.assets)
            .filter(|a| categories.contains(&a.category.as_str()) && a.faction.as_deref().is_none_or(|f| f == faction))
            .cloned().collect();
        egui::CollapsingHeader::new(format!("Bildmotive · {}", self.screen.name()))
            .id_salt(("bildmotive", self.screen as usize)).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for a in assets {
                    ui.vertical(|ui| {
                        ui.set_width(130.0);
                        self.motiv_datei(ui, &a, egui::vec2(126.0, 92.0));
                        ui.small(&a.label);
                    });
                }
            });
        });
    }

    pub(crate) fn motiv(&mut self, ui: &mut egui::Ui, category: &str, key: &str, view: &Value) {
        let faction = view["volk"].as_str().unwrap_or("").to_lowercase();
        let Some(catalog) = &self.catalog else { return; };
        let asset = select_asset(catalog, category, key, &faction).cloned();
        if let Some(asset) = asset { self.motiv_datei(ui, &asset, egui::vec2(112.0, 84.0)); }
    }

    pub(crate) fn motiv_datei(&mut self, ui: &mut egui::Ui, asset: &Asset, size: egui::Vec2) -> bool {
        self.motiv_anzeigen(ui, asset, size, true)
    }

    fn motiv_anzeigen(&mut self, ui: &mut egui::Ui, asset: &Asset, size: egui::Vec2, placeholder: bool) -> bool {
        let now = Instant::now();
        let check = self.art_textures.get(&asset.id).is_none_or(|entry| now.duration_since(entry.checked) >= Duration::from_secs(2));
        if check {
            let path = safe_relative(&asset.file).ok().map(|p| crate::root().join("content").join(p));
            let stamps = path.as_ref().map(|p| (stamp(p), stamp(&p.with_extension("provenance.json")))).unwrap_or((None, None));
            let changed = self.art_textures.get(&asset.id).is_none_or(|entry| entry.stamps != stamps || entry.texture.is_none());
            if !changed {
                self.art_textures.get_mut(&asset.id).unwrap().checked = now;
            } else {
            let loaded = path.and_then(|p| {
                let bytes = std::fs::read(&p).ok()?;
                let provenance: Value = serde_json::from_slice(&std::fs::read(p.with_extension("provenance.json")).ok()?).ok()?;
                if provenance["status"] != "complete" || provenance["profile"] != "final" || provenance["asset"] != asset.id || provenance["sha256"].as_str() != Some(&digest(&bytes)) { return None; }
                let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png).ok()?;
                if image.width() != asset.width || image.height() != asset.height { return None; }
                let image = image.thumbnail(768, 512).to_rgba8();
                Some(ui.ctx().load_texture(&asset.id, egui::ColorImage::from_rgba_unmultiplied(
                    [image.width() as usize, image.height() as usize], image.as_raw()), egui::TextureOptions::LINEAR))
            });
            self.art_textures.insert(asset.id.clone(), ArtCacheEntry { texture: loaded, stamps, checked: now });
            }
        }
        ui.ctx().request_repaint_after(Duration::from_secs(2));
        if let Some(texture) = self.art_textures.get(&asset.id).and_then(|entry| entry.texture.as_ref()) {
            let native = texture.size_vec2();
            let scale = (size.x / native.x).min(size.y / native.y);
            ui.image((texture.id(), native * scale)).on_hover_text(&asset.label);
            true
        } else {
            if placeholder { ui.allocate_ui(size, |ui| { ui.centered_and_justified(|ui| { ui.weak("Bild folgt").on_hover_text(&asset.label); }); }); }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn art_refreshes_after_creation_replacement_and_invalid_provenance() {
        let ctx = egui::Context::default();
        let mut app = Player::new(&ctx, crate::Session::new(42, 2, 0).unwrap());
        let mut asset = app.catalog.as_ref().unwrap().assets[0].clone();
        let unique = format!("ui-art-test-{}-{}", std::process::id(), SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_nanos());
        asset.id = unique.clone();
        asset.file = format!("runtime/{unique}.png");
        asset.width = 64;
        asset.height = 64;
        let path = crate::root().join("content").join(&asset.file);
        let proof = path.with_extension("provenance.json");
        let draw = |app: &mut Player| {
            if let Some(entry) = app.art_textures.get_mut(&asset.id) { entry.checked = Instant::now() - Duration::from_secs(3); }
            let mut shown = false;
            let _ = ctx.run(Default::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| { shown = app.motiv_datei(ui, &asset, egui::vec2(64.0, 64.0)); });
            });
            shown
        };
        let write = |color: [u8; 4], valid: bool| {
            image::RgbaImage::from_pixel(64, 64, image::Rgba(color)).save(&path).unwrap();
            let hash = if valid { digest(&std::fs::read(&path).unwrap()) } else { "incorrect".to_string() };
            std::fs::write(&proof, serde_json::to_vec(&serde_json::json!({"asset":asset.id,"status":"complete","profile":"final","sha256":hash})).unwrap()).unwrap();
        };
        assert!(!draw(&mut app), "missing artwork must remain unavailable");
        write([255, 0, 0, 255], true);
        assert!(draw(&mut app), "new image should load without restarting");
        let old_id = app.art_textures[&asset.id].texture.as_ref().unwrap().id();
        write([0, 255, 0, 255], false);
        assert!(!draw(&mut app), "a hash mismatch must hide stale art");
        write([0, 255, 0, 255], true);
        assert!(draw(&mut app), "publication should recover once provenance matches");
        assert_ne!(old_id, app.art_textures[&asset.id].texture.as_ref().unwrap().id());
        std::fs::remove_file(path).unwrap();
        std::fs::remove_file(proof).unwrap();
    }
    #[test]
    fn every_catalog_category_has_a_game_screen() {
        let catalog = Catalog::load(crate::root().join("content")).unwrap();
        for asset in &catalog.assets {
            assert!(Bildschirm::ALLE.into_iter().any(|s| screen_categories(s).contains(&asset.category.as_str())), "unwired: {}", asset.id);
        }
    }
    #[test]
    fn every_engine_object_resolves_for_every_faction() {
        let catalog = Catalog::load(crate::root().join("content")).unwrap();
        for volk in kern::Volk::ALLE {
            for g in kern::Gebaeude::ALLE {
                let a = select_asset(&catalog, "buildings", g.name(), volk.name()).unwrap();
                assert_eq!(a.faction.as_deref(), Some(volk.name()));
            }
            for e in kern::Einheit::ALLE {
                let a = select_asset(&catalog, if e.ist_schiff() { "ships" } else { "defenses" }, e.name(), volk.name()).unwrap();
                assert_eq!(a.faction.as_deref(), Some(volk.name()));
            }
            for f in kern::Forschung::ALLE { assert!(select_asset(&catalog, "research", f.name(), volk.name()).is_some()); }
            for m in kern::Mission::ALLE { assert!(select_asset(&catalog, "missions", m.name(), volk.name()).is_some()); }
        }
        assert!(select_asset(&catalog, "ships", "not_a_ship", "aurelianer").is_none());
    }
}
