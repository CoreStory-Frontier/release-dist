use anyhow::{anyhow, ensure, Context as _};
use chrono::{DateTime, Utc};
use semver::Version;
use serde::{ser, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use url::Url;

mod parse;
use parse::{Platform, PlatformKey, Product, ReleaseAsset};

pub struct Manifest {
    cli: Option<VersionContent>,
    desktop: Option<VersionContent>,
}

impl Manifest {
    pub fn from_dir(
        dist: &Path,
        base: &Url,
        tag: &str,
        notes: Option<String>,
        pub_date: DateTime<Utc>,
    ) -> anyhow::Result<Self> {
        let (mut cli_version, mut cli_platforms) = (None, BTreeMap::new());
        let (mut desktop_version, mut desktop_platforms) = (None, BTreeMap::new());

        fs::read_dir(dist)
            .with_context(|| format!("reading {}", dist.display()))?
            .try_for_each(|entry| {
                let path = entry?.path();
                let Some((asset, platform)) = Self::entry(&path, base, tag)? else {
                    return Ok::<_, anyhow::Error>(());
                };
                let (version, platforms) = match asset.product {
                    Product::Cli => (&mut cli_version, &mut cli_platforms),
                    Product::Desktop => (&mut desktop_version, &mut desktop_platforms),
                };
                if let Some(v) = asset.version {
                    let expected = version.get_or_insert_with(|| v.clone());
                    ensure!(
                        *expected == v,
                        "{} has version {v}, expected {expected}",
                        path.display()
                    );
                }
                let key = asset.platform;
                ensure!(
                    platforms.insert(key, platform).is_none(),
                    "found duplicate {} for key {key:?}",
                    path.display()
                );
                Ok(())
            })?;
        let cli = cli_version.map(|version| VersionContent {
            version,
            notes: notes.clone(),
            pub_date,
            platforms: cli_platforms,
        });
        let desktop = desktop_version.map(|version| VersionContent {
            version,
            notes,
            pub_date,
            platforms: desktop_platforms,
        });

        Ok(Self { cli, desktop })
    }

    pub fn write(&self, dir: &Path) -> anyhow::Result<()> {
        if let Some(cli) = self.cli.as_ref() {
            write_json(&dir.join("cli.json"), cli)?;
        }
        if let Some(desktop) = self.desktop.as_ref() {
            write_json(&dir.join("desktop.json"), desktop)?;
        }
        Ok(())
    }

    fn entry(
        path: &Path,
        base: &Url,
        tag: &str,
    ) -> anyhow::Result<Option<(ReleaseAsset, Platform)>> {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            return Ok(None);
        };
        let Ok((_, asset)) = ReleaseAsset::parse(name) else {
            return Ok(None);
        };
        let signature = match asset.product {
            Product::Cli => digest(&read(&sidecar(path, ".sha256"))?)?,
            Product::Desktop => read(&sidecar(path, ".sig"))?,
        };
        let platform = Platform {
            signature,
            url: asset_url(base, tag, name)?,
        };
        Ok(Some((asset, platform)))
    }
}

#[derive(Clone, Serialize)]
struct VersionContent {
    version: Version,
    notes: Option<String>,
    #[serde(serialize_with = "to_rfc3339")]
    pub_date: DateTime<Utc>,
    platforms: BTreeMap<PlatformKey, Platform>,
}

fn sidecar(path: &Path, suffix: &str) -> PathBuf {
    let mut s = path.as_os_str().to_owned();
    s.push(suffix);
    PathBuf::from(s)
}

fn digest(line: &str) -> anyhow::Result<String> {
    line.split_whitespace()
        .next()
        .map(str::to_owned)
        .ok_or_else(|| anyhow!("empty checksum"))
}

fn asset_url(base: &Url, tag: &str, file: &str) -> anyhow::Result<Url> {
    let mut url = base.clone();
    url.path_segments_mut()
        .map_err(|_| anyhow!("{base} cannot take a path"))?
        .pop_if_empty()
        .push(tag)
        .push(file);
    Ok(url)
}

fn read(path: &Path) -> anyhow::Result<String> {
    fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))
}

fn write_json(path: &Path, content: &VersionContent) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(content)?;
    fs::write(path, json).with_context(|| format!("writing {}", path.display()))
}

fn to_rfc3339<S: ser::Serializer>(d: &DateTime<Utc>, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&d.to_rfc3339())
}
