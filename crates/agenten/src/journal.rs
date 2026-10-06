use fs2::FileExt;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Single writer OS lock; an interrupted process releases the lock automatically.
/// Completed records are immutable. No pending request is ever retransmitted.
pub struct Journal {
    pub root: PathBuf,
    _lock: File,
}

impl Journal {
    pub fn open(path: &Path, manifest: &Value) -> Result<Self, String> {
        fs::create_dir_all(path).map_err(|e| e.to_string())?;
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path.join("writer.lock"))
            .map_err(|e| e.to_string())?;
        lock.try_lock_exclusive()
            .map_err(|_| "Ein anderer Prozess verwendet dieses Journal".to_string())?;
        let j = Self {
            root: path.to_path_buf(),
            _lock: lock,
        };
        j.put(
            "manifest.json",
            &serde_json::to_vec(manifest).map_err(|e| e.to_string())?,
        )?;
        fs::create_dir_all(path.join("calls")).map_err(|e| e.to_string())?;
        fs::create_dir_all(path.join("states")).map_err(|e| e.to_string())?;
        Ok(j)
    }

    pub fn put(&self, name: &str, bytes: &[u8]) -> Result<(), String> {
        let p = self.root.join(name);
        if p.exists() {
            return if fs::read(p).map_err(|e| e.to_string())? == bytes {
                Ok(())
            } else {
                Err(format!(
                    "Journal-Konflikt: {name}; eigener Laufordner nötig"
                ))
            };
        }
        let temp = p.with_extension("writing");
        let mut f = File::create(&temp).map_err(|e| e.to_string())?;
        f.write_all(bytes)
            .and_then(|_| f.sync_all())
            .map_err(|e| e.to_string())?;
        drop(f);
        fs::rename(temp, p).map_err(|e| e.to_string())
    }

    pub fn read(&self, name: &str) -> Result<Option<Vec<u8>>, String> {
        match fs::read(self.root.join(name)) {
            Ok(b) => Ok(Some(b)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn request_count(&self) -> Result<usize, String> {
        let mut n = 0;
        for p in fs::read_dir(self.root.join("calls")).map_err(|e| e.to_string())? {
            if p.map_err(|e| e.to_string())?
                .file_name()
                .to_string_lossy()
                .ends_with(".request.json")
            {
                n += 1;
            }
        }
        Ok(n)
    }

    pub fn reserve(&self, key: &str, request: &Value, max: usize) -> Result<Option<Value>, String> {
        let request_name = format!("calls/{key}.request.json");
        let response_name = format!("calls/{key}.response.json");
        let bytes = serde_json::to_vec(request).map_err(|e| e.to_string())?;
        if let Some(old) = self.read(&request_name)? {
            if old != bytes {
                return Err(format!("Eingaben für {key} haben sich geändert"));
            }
            return match self.read(&response_name)? {
                Some(b) => Ok(Some(serde_json::from_slice(&b).map_err(|e| format!("Journal beschädigt: {e}"))?)),
                None => Err(format!("{key}: offener Aufruf mit unbekanntem Ausgang; automatische Wiederholung gesperrt")),
            };
        }
        if self.request_count()? >= max {
            return Err("Persistente Anfragegrenze erreicht".into());
        }
        self.put(&request_name, &bytes)?;
        Ok(None)
    }

    /// Withdraws a reservation whose call the provider provably refused (HTTP 401/402/403):
    /// it was not processed, so sending it again later is not a duplicate.
    pub fn cancel(&self, key: &str) -> Result<(), String> {
        if self.root.join(format!("calls/{key}.response.json")).exists() {
            return Err(format!("{key}: abgeschlossener Aufruf kann nicht zurückgezogen werden"));
        }
        fs::remove_file(self.root.join(format!("calls/{key}.request.json")))
            .map_err(|e| e.to_string())
    }

    pub fn finish(&self, key: &str, response: &Value) -> Result<(), String> {
        self.put(
            &format!("calls/{key}.response.json"),
            &serde_json::to_vec(response).map_err(|e| e.to_string())?,
        )
    }

    pub fn save_world(&self, index: usize, world: &kern::Welt) -> Result<(), String> {
        let data = world.zu_bytes();
        self.put(&format!("states/{index:08}.bin"), &data)?;
        let marker = serde_json::json!({"sha256":hash(&data), "world_hash":world.hash()});
        self.put(
            &format!("states/{index:08}.json"),
            &serde_json::to_vec(&marker).unwrap(),
        )
    }

    pub fn load_world(&self) -> Result<Option<(usize, kern::Welt)>, String> {
        let mut indices = Vec::new();
        for item in fs::read_dir(self.root.join("states")).map_err(|e| e.to_string())? {
            let path = item.map_err(|e| e.to_string())?.path();
            if path.extension().and_then(|x| x.to_str()) == Some("json") {
                if let Some(n) = path
                    .file_stem()
                    .and_then(|x| x.to_str())
                    .and_then(|x| x.parse::<usize>().ok())
                {
                    indices.push(n);
                }
            }
        }
        let Some(index) = indices.into_iter().max() else {
            return Ok(None);
        };
        let data = self
            .read(&format!("states/{index:08}.bin"))?
            .ok_or("Checkpoint fehlt")?;
        let marker: Value = serde_json::from_slice(
            &self
                .read(&format!("states/{index:08}.json"))?
                .ok_or("Checkpoint-Marker fehlt")?,
        )
        .map_err(|e| e.to_string())?;
        if marker["sha256"] != hash(&data) {
            return Err("Checkpoint-Prüfsumme falsch".into());
        }
        let world = kern::Welt::aus_bytes(&data)?;
        if marker["world_hash"] != world.hash() {
            return Err("Checkpoint-Zustandshash falsch".into());
        }
        Ok(Some((index, world)))
    }
}
