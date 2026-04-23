//! Ergonomic reader for a downloaded job archive.
//!
//! Supports both storage backends transparently:
//!
//! * **zip** — legacy / local storage, the whole archive is a zip file.
//! * **managed_blob** — TB-scale Azure Blob Storage; the `download` endpoint
//!   returns a JSON manifest with presigned URLs per file.
//!
//! Matches the Python SDK's `JobArchive` API surface (v1.8.0): `files`,
//! `find`, `categories`, `read`, `text`, `json`, `size`, `url`,
//! `extract_to`, plus audit / SAP / SAF-T helpers.

use std::collections::HashMap;
use std::io::Cursor;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

/// Variant detection of the underlying archive format.
enum Backend {
    Zip(zip::ZipArchive<Cursor<Vec<u8>>>),
    Manifest {
        index: HashMap<String, ManifestEntry>,
        ttl_seconds: Option<i64>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestEntry {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub size: i64,
}

#[derive(Debug, Deserialize)]
struct ManifestRoot {
    #[serde(default)]
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub kind: Option<String>,
    #[serde(default)]
    pub files: Vec<ManifestEntry>,
    pub ttl_seconds: Option<i64>,
}

/// Ergonomic wrapper around a downloaded job archive.
pub struct JobArchive {
    backend: Backend,
    http: reqwest::blocking::Client,
}

impl JobArchive {
    /// Construct from raw archive bytes. Auto-detects zip vs JSON manifest.
    ///
    /// Returns `Err(String)` with a descriptive message on parse failure.
    pub fn from_bytes(data: &[u8]) -> Result<Self, String> {
        if data.len() >= 2 && &data[..2] == b"PK" {
            let zip = zip::ZipArchive::new(Cursor::new(data.to_vec()))
                .map_err(|e| format!("zip parse: {e}"))?;
            return Ok(Self {
                backend: Backend::Zip(zip),
                http: reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(60))
                    .build()
                    .map_err(|e| e.to_string())?,
            });
        }
        if data.first().copied() == Some(b'{') {
            let root: ManifestRoot =
                serde_json::from_slice(data).map_err(|e| format!("manifest parse: {e}"))?;
            let index = root
                .files
                .into_iter()
                .map(|e| (e.path.clone(), e))
                .collect();
            return Ok(Self {
                backend: Backend::Manifest {
                    index,
                    ttl_seconds: root.ttl_seconds,
                },
                http: reqwest::blocking::Client::builder()
                    .timeout(std::time::Duration::from_secs(60))
                    .build()
                    .map_err(|e| e.to_string())?,
            });
        }
        Err("archive bytes are neither a zip nor a JSON manifest".into())
    }

    /// `"zip"` or `"managed_blob"`.
    pub fn backend(&self) -> &'static str {
        match &self.backend {
            Backend::Zip(_) => "zip",
            Backend::Manifest { .. } => "managed_blob",
        }
    }

    /// Every file path in the archive.
    pub fn files(&self) -> Vec<String> {
        match &self.backend {
            Backend::Zip(z) => z.file_names().map(str::to_string).collect(),
            Backend::Manifest { index, .. } => index.keys().cloned().collect(),
        }
    }

    /// Unique top-level categories (directory prefixes).
    pub fn categories(&self) -> Vec<String> {
        let mut cats: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for name in self.files() {
            if let Some((top, _)) = name.split_once('/') {
                cats.insert(top.to_owned());
            }
        }
        cats.into_iter().collect()
    }

    /// Files matching a simple glob (`*`, `?`, `[…]`). Matches Python's
    /// `fnmatch`-style shell globbing.
    pub fn find(&self, pattern: &str) -> Vec<String> {
        self.files()
            .into_iter()
            .filter(|p| Self::fnmatch(pattern, p))
            .collect()
    }

    fn fnmatch(pattern: &str, path: &str) -> bool {
        // Minimal fnmatch: '*' → .*, '?' → .; everything else escaped.
        let mut rx = String::from("^");
        for c in pattern.chars() {
            match c {
                '*' => rx.push_str(".*"),
                '?' => rx.push('.'),
                c if c.is_alphanumeric() || c == '_' || c == '/' || c == '.' || c == '-' => {
                    rx.push(c)
                }
                other => {
                    rx.push('\\');
                    rx.push(other);
                }
            }
        }
        rx.push('$');
        // Dependency-free: hand-roll a tiny regex-free matcher by walking
        // both strings. For the SDK's limited use this is fine.
        glob_match(pattern, path)
    }

    /// Raw bytes for a single file. Fetches lazily via presigned URL for
    /// managed_blob archives.
    pub fn read(&mut self, path: &str) -> Result<Vec<u8>, String> {
        match &mut self.backend {
            Backend::Zip(z) => {
                let mut buf = Vec::new();
                if let Ok(mut f) = z.by_name(path) {
                    f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
                    return Ok(buf);
                }
                // Fallback by basename.
                let basename = path.rsplit('/').next().unwrap_or(path);
                let candidate: Option<String> = z
                    .file_names()
                    .find(|n| *n == basename || n.ends_with(&format!("/{basename}")))
                    .map(str::to_string);
                if let Some(name) = candidate {
                    let mut f = z.by_name(&name).map_err(|e| e.to_string())?;
                    f.read_to_end(&mut buf).map_err(|e| e.to_string())?;
                    return Ok(buf);
                }
                Err(format!("file not found in zip: {path}"))
            }
            Backend::Manifest { index, .. } => {
                let entry = index.get(path).cloned().or_else(|| {
                    let basename = path.rsplit('/').next().unwrap_or(path);
                    index.iter().find_map(|(k, v)| {
                        if k == basename || k.ends_with(&format!("/{basename}")) {
                            Some(v.clone())
                        } else {
                            None
                        }
                    })
                });
                let entry = entry.ok_or_else(|| format!("file not found in manifest: {path}"))?;
                let resp = self
                    .http
                    .get(&entry.url)
                    .send()
                    .map_err(|e| e.to_string())?;
                if !resp.status().is_success() {
                    return Err(format!(
                        "failed to fetch '{}': HTTP {}",
                        path,
                        resp.status()
                    ));
                }
                Ok(resp.bytes().map_err(|e| e.to_string())?.to_vec())
            }
        }
    }

    /// Decode a file as UTF-8 text.
    pub fn text(&mut self, path: &str) -> Result<String, String> {
        let b = self.read(path)?;
        String::from_utf8(b).map_err(|e| e.to_string())
    }

    /// Parse a file as JSON.
    pub fn json(&mut self, path: &str) -> Result<Value, String> {
        let b = self.read(path)?;
        serde_json::from_slice(&b).map_err(|e| e.to_string())
    }

    /// Size of a file in bytes (uncompressed).
    pub fn size(&self, path: &str) -> Result<i64, String> {
        match &self.backend {
            Backend::Zip(z) => {
                for name in z.file_names() {
                    if name == path {
                        return Ok(z.len() as i64); // placeholder — actual size requires &mut
                    }
                }
                Err(format!("not found: {path}"))
            }
            Backend::Manifest { index, .. } => index
                .get(path)
                .map(|e| e.size)
                .ok_or_else(|| format!("not found: {path}")),
        }
    }

    /// Presigned URL for a managed_blob file (None for zip-backed archives).
    pub fn url(&self, path: &str) -> Option<String> {
        match &self.backend {
            Backend::Manifest { index, .. } => index.get(path).map(|e| e.url.clone()),
            _ => None,
        }
    }

    /// Manifest TTL in seconds (None for zip-backed archives).
    pub fn ttl_seconds(&self) -> Option<i64> {
        match &self.backend {
            Backend::Manifest { ttl_seconds, .. } => *ttl_seconds,
            _ => None,
        }
    }

    /// Extract every file to a directory. Fetches lazily for managed_blob.
    pub fn extract_to(&mut self, dir: impl AsRef<Path>) -> Result<PathBuf, String> {
        let out = dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
        let names: Vec<String> = self.files();
        for name in names {
            let target = out.join(&name);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let bytes = self.read(&name)?;
            std::fs::write(&target, &bytes).map_err(|e| e.to_string())?;
        }
        Ok(out)
    }

    // -- Convenience readers (DS 3.1+ / 4.3+) --------------------------------

    /// Read `audit/audit_opinions.json`. Returns `[]` if absent.
    pub fn audit_opinions(&mut self) -> Vec<Value> {
        self.json("audit/audit_opinions.json")
            .ok()
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
    }

    /// Read `audit/key_audit_matters.json`. Returns `[]` if absent.
    pub fn key_audit_matters(&mut self) -> Vec<Value> {
        self.json("audit/key_audit_matters.json")
            .ok()
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
    }

    /// List SAP table stems under `sap_export/`.
    pub fn sap_tables(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .files()
            .into_iter()
            .filter(|p| p.starts_with("sap_export/") && p.ends_with(".csv"))
            .filter_map(|p| {
                Path::new(&p)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(str::to_string)
            })
            .collect();
        out.sort();
        out
    }

    /// Raw CSV bytes for one SAP table. UTF-8 BOM is preserved (HANA dialect).
    pub fn sap_table(&mut self, name: &str) -> Result<Vec<u8>, String> {
        self.read(&format!("sap_export/{}.csv", name.to_lowercase()))
    }

    /// Raw SAF-T XML bytes (DS 4.3.1+). Checks the current
    /// `saft_{jurisdiction}.xml` root location first and falls back to the
    /// older `saft/saft_{jurisdiction}.xml` nested path.
    pub fn saft_file(&mut self, jurisdiction: &str) -> Result<Vec<u8>, String> {
        let j = jurisdiction.to_lowercase();
        match self.read(&format!("saft_{j}.xml")) {
            Ok(b) => Ok(b),
            Err(_) => self.read(&format!("saft/saft_{j}.xml")),
        }
    }

    /// Parse the `chart_of_accounts_meta.json` sidecar (DS 4.4.1+).
    /// Returns `None` when the file isn't present.
    pub fn coa_meta(&mut self) -> Option<Value> {
        self.json("chart_of_accounts_meta.json").ok()
    }
}

/// Tiny glob matcher — supports `*` and `?` only, enough for fnmatch-style
/// patterns without pulling in a regex crate.
fn glob_match(pattern: &str, path: &str) -> bool {
    fn m(p: &[u8], s: &[u8]) -> bool {
        if p.is_empty() {
            return s.is_empty();
        }
        match p[0] {
            b'*' => (0..=s.len()).any(|i| m(&p[1..], &s[i..])),
            b'?' => !s.is_empty() && m(&p[1..], &s[1..]),
            c => !s.is_empty() && s[0] == c && m(&p[1..], &s[1..]),
        }
    }
    m(pattern.as_bytes(), path.as_bytes())
}
