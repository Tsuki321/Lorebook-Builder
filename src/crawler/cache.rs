use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::pages::PageData;

pub struct Cache {
    dir: PathBuf,
    index: HashMap<String, (PathBuf, u64)>,
}

impl Cache {
    pub fn open(dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(dir).ok();
        let mut s = Self { dir: dir.to_path_buf(), index: HashMap::new() };
        s.load_index()?;
        Ok(s)
    }

    fn load_index(&mut self) -> Result<()> {
        let idx = self.dir.join("index.txt");
        if !idx.exists() { return Ok(()); }
        let s = std::fs::read_to_string(&idx).unwrap_or_default();
        for line in s.lines() {
            if let Some((k, rest)) = line.split_once('\t') {
                if let Some((path, revid)) = rest.split_once('\t') {
                    if let Ok(r) = revid.parse::<u64>() {
                        self.index.insert(k.to_string(), (PathBuf::from(path), r));
                    }
                }
            }
        }
        Ok(())
    }

    fn save_index(&self) -> Result<()> {
        let idx = self.dir.join("index.txt");
        let mut s = String::new();
        for (k, (p, r)) in &self.index {
            s.push_str(k);
            s.push('\t');
            s.push_str(&p.to_string_lossy());
            s.push('\t');
            s.push_str(&r.to_string());
            s.push('\n');
        }
        std::fs::write(idx, s).context("writing cache index")?;
        Ok(())
    }

    pub fn dir(&self) -> &Path { &self.dir }

    pub fn get(&self, title: &str) -> Result<Option<PageData>> {
        let Some((path, _revid)) = self.index.get(title) else { return Ok(None); };
        if !path.exists() { return Ok(None); }
        let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
        let page: PageData = serde_json::from_slice(&bytes)
            .with_context(|| format!("parsing cached page {}", path.display()))?;
        Ok(Some(page))
    }

    pub fn put(&mut self, title: &str, page: &PageData) -> Result<()> {
        let safe = sanitize(title);
        let path = self.dir.join(format!("{}-{}.json", safe, page.revid));
        let bytes = serde_json::to_vec(page)?;
        std::fs::write(&path, bytes)
            .with_context(|| format!("writing cache file {}", path.display()))?;
        self.index.insert(title.to_string(), (path, page.revid));
        self.save_index()?;
        Ok(())
    }
}

fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}
