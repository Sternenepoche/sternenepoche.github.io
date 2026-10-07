//! Simulationsschleife: Ereigniswarteschlange, Entscheidungsfenster, Wecklogik, Zustandshash.

use crate::typen::*;
use crate::welt::*;
use sha2::{Digest, Sha256};
use std::collections::BinaryHeap;

impl Welt {
    /// Interactive/model clients must finish every due role before advancing.
    /// Offline replay and baseline bots retain the lower-level `schritt` primitive.
    pub fn schritt_wenn_bereit(&mut self) -> Result<bool, String> {
        if self
            .faellig
            .iter()
            .any(|f| self.spieler[f.spieler as usize].weck[f.rolle.idx()].letzter < self.zeit)
        {
            return Err("Entscheidungsfenster wartet noch auf Rollen-Antworten".into());
        }
        Ok(self.schritt())
    }

    fn zeit_vorruecken(&mut self, ziel: SimZeit) {
        let dt = ziel - self.zeit;
        if dt > 0 {
            let grenze = milli(self.regeln.stabilitaet.unruhen_unter);
            for p in &mut self.planeten {
                if p.stabilitaet < grenze {
                    if let Some(auftrag) = p.bauschleife.first_mut() {
                        if let Some(fertig) = &mut auftrag.fertig {
                            *fertig += dt;
                        }
                    }
                }
            }
        }
        self.zeit = ziel;
    }
    pub fn beendet(&self) -> bool {
        self.zeit >= self.regeln.epochenende()
    }

    /// Rückt die Welt um ein Entscheidungsfenster vor. Während die Agenten entscheiden,
    /// steht die Weltuhr still: gerechnet wird nur hier.
    pub fn schritt(&mut self) -> bool {
        if self.beendet() {
            return false;
        }
        self.kolonisation_fensterabschluss();
        let ziel = (self.zeit + self.regeln.fenster()).min(self.regeln.epochenende());
        while self
            .ereignisse
            .peek()
            .map(|e| e.zeit <= ziel)
            .unwrap_or(false)
        {
            let e = self.ereignisse.pop().unwrap();
            self.zeit_vorruecken(e.zeit);
            self.ausfuehren(e.art);
        }
        self.zeit_vorruecken(ziel);
        if self.aufklaerungsregeln() {
            self.aufklaerung.abschirmung.retain(|id,_|self.flotten.contains_key(id));
            self.aufklaerung.sondenziele.retain(|id,_|self.flotten.contains_key(id));
        }
        if self.beendet() {
            self.punkte_neu();
        }
        self.fenster_vorbereiten();
        true
    }

    fn ausfuehren(&mut self, art: EreignisArt) {
        match art {
            EreignisArt::SensorKontakt { flotte } => self.sensor_kontakt(flotte),
            EreignisArt::Tick => {
                self.tick();
                self.plane(self.zeit + STUNDE, EreignisArt::Tick);
            }
            EreignisArt::Tag => {
                self.tageswechsel();
                self.plane(self.zeit + TAG, EreignisArt::Tag);
            }
            EreignisArt::BauFertig { planet } => self.bau_fertig(planet as usize),
            EreignisArt::FertigungFertig { planet, schleife } => {
                self.fertigung_fertig(planet as usize, schleife as usize)
            }
            EreignisArt::FlotteAnkunft { flotte } => self.flotte_ankunft(flotte),
            EreignisArt::FlotteRueckkehr { flotte } => self.flotte_rueckkehr(flotte),
            EreignisArt::OrbitEnde { flotte } => self.orbit_ende(flotte),
            EreignisArt::Marktlieferung { planet, gut, menge } => {
                self.marktlieferung(planet as usize, gut, menge)
            }
            EreignisArt::MarktlieferungGebunden {
                planet,
                empfaenger,
                gut,
                menge,
            } => self.marktlieferung_gebunden(planet as usize, empfaenger, gut, menge),
            EreignisArt::VertragEnde { vertrag } => self.vertrag_ende(vertrag),
            EreignisArt::RaketenFertig {
                planet,
                besitzer,
                art,
                anzahl,
            } => self.raketen_fertig(planet as usize, besitzer, art, anzahl),
            EreignisArt::RaketenAnkunft {
                von,
                ziel,
                anzahl,
                zieltyp,
            } => self.raketen_ankunft(von, ziel, anzahl, zieltyp),
        }
    }

    /// Lost die Reihenfolge der Spieler für dieses Fenster aus und bestimmt die fälligen Rollen.
    pub fn fenster_vorbereiten(&mut self) {
        let r = self.regeln.clone();
        let jetzt = self.zeit;
        let fenster_nr = (jetzt / r.fenster()) as u64;
        let mut rng = strom(self.startwert, KANAL_REIHENFOLGE, fenster_nr);
        let mut reihenfolge: Vec<SpielerId> = (0..self.spieler.len() as SpielerId).collect();
        mischen(&mut rng, &mut reihenfolge);

        // Feindliche Flotten, die neu in den Sensorbereich kommen, wecken den Feldherrn sofort.
        let mut warnungen: Vec<(FlottenId, SpielerId)> = Vec::new();
        for f in self.flotten.values() {
            if f.gemeldet || !f.mission.feindlich() || f.zustand != Flottenzustand::Hinflug {
                continue;
            }
            if let Some(zp) = self.belegung.get(&f.ziel) {
                if self.aufklaerungsregeln() && (f.besitzer==self.planeten[*zp as usize].besitzer || self.verbuendet(f.besitzer,self.planeten[*zp as usize].besitzer)){continue;}
                if jetzt >= f.ankunft - self.warnzeit(*zp as usize) {
                    warnungen.push((f.id, self.planeten[*zp as usize].besitzer));
                }
            }
        }
        for (fid, ziel) in warnungen {
            self.flotten.get_mut(&fid).unwrap().gemeldet = true;
            self.wecke(ziel, Rolle::Feldherr, "Feindliche Flotte im Anflug", true);
            for sid in 0..self.spieler.len() as SpielerId {
                if self.verbuendet(sid, ziel) {
                    self.wecke(
                        sid,
                        Rolle::Feldherr,
                        "Feindliche Flotte im Anflug auf einen Partner",
                        true,
                    );
                }
            }
        }
        // Leere Bauschleifen wecken den Verwalter, je Leerstand einmal.
        for pid in 0..self.planeten.len() {
            let p = &self.planeten[pid];
            if p.bauschleife.is_empty() && !p.leer_gemeldet {
                let besitzer = p.besitzer;
                self.planeten[pid].leer_gemeldet = true;
                self.wecke(besitzer, Rolle::Verwalter, "Bauschleife leer", false);
            }
        }

        let mut faellig = Vec::new();
        for sid in &reihenfolge {
            let sp = &self.spieler[*sid as usize];
            if !sp.ki || self.beendet() {
                continue;
            }
            for rolle in [
                Rolle::Stratege,
                Rolle::Verwalter,
                Rolle::Feldherr,
                Rolle::Diplomat,
            ] {
                let w = &sp.weck[rolle.idx()];
                let takt = r.agenten.takt_stunden[&rolle] * STUNDE;
                let mut gruende: Vec<String> = Vec::new();
                if jetzt - w.letzter >= takt {
                    gruende.push("Regeltakt".into());
                }
                if w.wecker.map(|t| t <= jetzt).unwrap_or(false) {
                    gruende.push("Wecker".into());
                }
                if !w.ausloeser.is_empty()
                    && (w.dringend || jetzt - w.letzter >= r.agenten.frueheste_sekunden(rolle))
                {
                    gruende.extend(w.ausloeser.iter().cloned());
                }
                if !gruende.is_empty() {
                    faellig.push(Faellig {
                        spieler: *sid,
                        name: sp.name.clone(),
                        rolle,
                        gruende,
                    });
                }
            }
        }
        self.reihenfolge = reihenfolge;
        self.faellig = faellig;
    }

    /// SHA-256 über den gesamten Zustand. Zwei Läufe mit gleichem Startwert und
    /// gleichem Aktionsprotokoll liefern denselben Hash.
    pub fn hash(&self) -> String {
        crate::snapshot_layout::with_layout(!self.aufklaerung.neues_layout, || self.hash_intern())
    }
    fn hash_intern(&self) -> String {
        let mut kopie = self.clone();
        let ereignisse =
            std::mem::replace(&mut kopie.ereignisse, BinaryHeap::new()).into_sorted_vec();
        let mut h = Sha256::new();
        h.update(bincode::serialize(&kopie).expect("Zustand serialisierbar"));
        h.update(bincode::serialize(&ereignisse).expect("Ereignisse serialisierbar"));
        if self.kolonisation.aktiv {
            if self.kolonisationsregeln_v2() {
                h.update(b"kolonisation-v2");
                h.update(bincode::serialize(&self.kolonisation).expect("Kolonisationszustand"));
            } else {
                h.update(b"kolonisation-v1");
                h.update(
                    bincode::serialize(&crate::kolonisation::KolonisationszustandV1::from(
                        &self.kolonisation,
                    ))
                    .expect("Kolonisationszustand"),
                );
            }
        }
        if !self.scans.is_empty() {
            h.update(b"sternkarte-v1");
            h.update(bincode::serialize(&self.scans).expect("Scans"));
        }
        if self.aufklaerung.neues_layout {
            h.update(b"aufklaerung-online-v1");
            h.update(bincode::serialize(&self.aufklaerung).expect("Aufklärungszustand"));
        }
        h.finalize().iter().map(|b| format!("{b:02x}")).collect()
    }

    /// Vollständiger Schnappschuss als Bytes.
    pub fn zu_bytes(&self) -> Vec<u8> {
        let base = crate::snapshot_layout::with_layout(!self.aufklaerung.neues_layout, || self.zu_bytes_mit_scans());
        if !self.aufklaerung.neues_layout { return base; }
        let mut out = b"STERNEP6".to_vec();
        out.extend_from_slice(&(base.len() as u64).to_le_bytes());
        out.extend(base);
        out.extend(bincode::serialize(&self.aufklaerung).expect("Aufklärungszustand"));
        out
    }
    fn zu_bytes_mit_scans(&self) -> Vec<u8> {
        let base = self.zu_bytes_legacy();
        if self.scans.is_empty() { return base; }
        let mut out = b"STERNEP5".to_vec();
        out.extend_from_slice(&(base.len() as u64).to_le_bytes());
        out.extend(base);
        out.extend(bincode::serialize(&self.scans).expect("Scans"));
        out
    }

    fn zu_bytes_legacy(&self) -> Vec<u8> {
        let base = bincode::serialize(self).expect("Zustand serialisierbar");
        if !self.kolonisation.aktiv {
            return base;
        }
        let mut out = if self.kolonisationsregeln_v2() {
            b"STERNEP4".to_vec()
        } else {
            b"STERNEP3".to_vec()
        };
        out.extend_from_slice(&(base.len() as u64).to_le_bytes());
        out.extend(base);
        if self.kolonisationsregeln_v2() {
            out.extend(bincode::serialize(&self.kolonisation).expect("Kolonisationszustand"));
        } else {
            out.extend(
                bincode::serialize(&crate::kolonisation::KolonisationszustandV1::from(
                    &self.kolonisation,
                ))
                .expect("Kolonisationszustand"),
            );
        }
        out
    }

    pub fn aus_bytes(b: &[u8]) -> Result<Welt, String> {
        if b.starts_with(b"STERNEP6") {
            let len = u64::from_le_bytes(b.get(8..16).ok_or("Snapshotkopf fehlt")?.try_into().unwrap()) as usize;
            let end = 16usize.checked_add(len).filter(|n|*n<=b.len()).ok_or("Snapshotlänge ungültig")?;
            if b[16..end].starts_with(b"STERNEP6") { return Err("Verschachtelter V6-Snapshot".into()); }
            let mut w = crate::snapshot_layout::with_layout(false, ||Self::aus_bytes_intern(&b[16..end]))?;
            w.aufklaerung = bincode::deserialize(&b[end..]).map_err(|e|e.to_string())?;
            if !w.aufklaerung.neues_layout { return Err("V6-Snapshot ohne erweitertes Layout".into()); }
            return Ok(w);
        }
        crate::snapshot_layout::with_layout(true, ||Self::aus_bytes_intern(b))
    }
    fn aus_bytes_intern(b: &[u8]) -> Result<Welt, String> {
        if b.starts_with(b"STERNEP5") {
            let len = u64::from_le_bytes(b.get(8..16).ok_or("Snapshotkopf fehlt")?.try_into().unwrap()) as usize;
            let end = 16usize.checked_add(len).filter(|n| *n <= b.len()).ok_or("Snapshotlänge ungültig")?;
            if b[16..end].starts_with(b"STERNEP5") { return Err("Verschachtelter V5-Snapshot".into()); }
            let mut w = Self::aus_bytes_intern(&b[16..end])?;
            w.scans = bincode::deserialize(&b[end..]).map_err(|e| e.to_string())?;
            return Ok(w);
        }
        if b.starts_with(b"STERNEP3") || b.starts_with(b"STERNEP4") {
            let len = u64::from_le_bytes(
                b.get(8..16)
                    .ok_or("Unvollständiger Snapshotkopf")?
                    .try_into()
                    .unwrap(),
            ) as usize;
            let end = 16usize
                .checked_add(len)
                .filter(|end| *end <= b.len())
                .ok_or("Snapshotlänge ungültig")?;
            let mut w: Welt = bincode::deserialize(&b[16..end]).map_err(|e| e.to_string())?;
            if b.starts_with(b"STERNEP4") {
                w.kolonisation = bincode::deserialize(&b[end..]).map_err(|e| e.to_string())?;
                if !w.kolonisationsregeln_v2() {
                    return Err("V4-Snapshot benötigt Kolonisationsregeln v2".into());
                }
            } else {
                let old: crate::kolonisation::KolonisationszustandV1 =
                    bincode::deserialize(&b[end..]).map_err(|e| e.to_string())?;
                w.kolonisation = old.into();
            }
            return Ok(w);
        }
        bincode::deserialize(b).map_err(|e| format!("Schnappschuss nicht lesbar: {e}"))
    }

    /// Holt die seit dem letzten Aufruf angefallenen Protokolleinträge ab.
    pub fn log_abholen(&mut self) -> Vec<Logeintrag> {
        std::mem::take(&mut self.logpuffer)
    }
}
