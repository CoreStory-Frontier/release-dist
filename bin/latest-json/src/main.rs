use anyhow::{Context as _, anyhow, bail};
use chrono::{DateTime, Utc};
use clap::Parser;
use semver::Version;
use serde::de::IntoDeserializer as _;
use serde::de::value::{Error as ValueError, StrDeserializer};
use serde::{Deserialize, Serialize, ser};
use std::collections::BTreeMap;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};
use url::Url;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut desktop = Manifest::default();
    let mut cli = Manifest::default();

    fs::read_dir(&args.dist)
        .with_context(|| format!("reading {}", args.dist.display()))?
        .into_iter()
        .try_for_each(|entry| {
            let Some(e) = ManifestEntry::new(&args, entry?)? else {
                return Ok::<_, anyhow::Error>(());
            };
            match e {
                ManifestEntry::Desktop(f, v, k, p) => {
                    desktop.insert(&f, k, v, p)?
                },
                ManifestEntry::Cli(f, v, k, p) => cli.insert(&f, k, v, p)?,
            }
            Ok(())
        })?;

    let pub_date = Utc::now();

    write(
        &args.dist.join("desktop.json"),
        &desktop.finish("desktop", args.notes.clone(), pub_date)?,
    )?;
    write(
        &args.dist.join("cli.json"),
        &cli.finish("CLI", args.notes, pub_date)?,
    )?;

    Ok(())
}

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    tag: String,
    #[arg(long)]
    base_url: Url,
    #[arg(long, default_value = "dist")]
    dist: PathBuf,
    #[arg(long)]
    notes: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct VersionContent {
    version: Version,
    notes: Option<String>,
    #[serde(serialize_with = "to_rfc3339")]
    pub_date: DateTime<Utc>,
    platforms: BTreeMap<PlatformKey, Platform>,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
enum PlatformKey {
    #[serde(rename = "aarch64-apple-darwin")]
    DarwinArm,
    #[serde(rename = "x86_64-unknown-linux-gnu")]
    LinuxX64,
    #[serde(rename = "x86_64-unknown-linux-musl")]
    LinuxMusl,
    #[serde(rename = "x86_64-pc-windows-gnu")]
    WindowsX64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Platform {
    signature: String,
    url: url::Url,
}

enum ManifestEntry {
    Desktop(String, Version, PlatformKey, Platform),
    Cli(String, Version, PlatformKey, Platform),
}

impl ManifestEntry {
    fn new(args: &Args, entry: DirEntry) -> anyhow::Result<Option<Self>> {
        let path = entry.path();

        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            return Ok(None);
        };

        if let Some(bundle) = name.strip_suffix(".sig") {
            let (key, version) = parse_desktop(bundle)?;
            require_file(&args.dist.join(bundle))?;
            let signature = read(&path)?;
            let url = asset_url(&args.base_url, &args.tag, bundle)?;
            let file = name.to_string();
            let platform = Platform { signature, url };
            let this = Self::Desktop(file, version, key, platform);
            Ok(Some(this))
        } else if let Some(stem) = name
            .strip_prefix("corestory-cli-")
            .and_then(|r| r.strip_suffix(".tar.gz"))
        {
            let (key, version) = parse_cli(stem)
                .with_context(|| format!("{name} is not <version>-<target>"))?;
            let signature =
                digest(&read(&args.dist.join(format!("{name}.sha256")))?)
                    .with_context(|| format!("{name}.sha256 is empty"))?;
            let url = asset_url(&args.base_url, &args.tag, name)?;
            let platform = Platform { signature, url };
            let file = name.to_string();
            let this = Self::Cli(file, version, key, platform);
            Ok(Some(this))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Default)]
struct Manifest {
    version: Option<Version>,
    platforms: BTreeMap<PlatformKey, Platform>,
}

impl Manifest {
    fn insert(
        &mut self,
        file: &str,
        key: PlatformKey,
        version: Version,
        platform: Platform,
    ) -> anyhow::Result<()> {
        match &self.version {
            Some(expected) => {
                if *expected != version {
                    bail!("{file} has version {version}, expected {expected}")
                }
            },
            None => self.version = Some(version),
        }
        if self.platforms.insert(key, platform).is_some() {
            bail!("{file} is a second asset for {key:?}");
        }
        Ok(())
    }

    fn finish(
        self,
        what: &str,
        notes: Option<String>,
        pub_date: DateTime<Utc>,
    ) -> anyhow::Result<VersionContent> {
        let version =
            self.version.ok_or_else(|| anyhow!("no {what} assets found"))?;
        Ok(VersionContent {
            version,
            notes,
            pub_date,
            platforms: self.platforms,
        })
    }
}

fn parse_desktop(bundle: &str) -> anyhow::Result<(PlatformKey, Version)> {
    let (stem, key) =
        if let Some(stem) = bundle.strip_suffix("_aarch64.app.tar.gz") {
            (stem, PlatformKey::DarwinArm)
        } else if let Some(stem) = bundle.strip_suffix("_amd64.deb") {
            (stem, PlatformKey::LinuxX64)
        } else {
            bail!("{bundle} is not a known desktop updater bundle");
        };
    let (_product, version) = stem
        .split_once('_')
        .with_context(|| format!("{bundle} has no version"))?;
    let version = version
        .parse()
        .with_context(|| format!("{bundle} has version {version}"))?;
    Ok((key, version))
}

fn parse_cli(stem: &str) -> Option<(PlatformKey, Version)> {
    stem.match_indices('-').find_map(|(at, _)| {
        let triple: StrDeserializer<'_, ValueError> =
            stem[at + 1..].into_deserializer();
        let key = PlatformKey::deserialize(triple).ok()?;
        let version = stem[..at].parse().ok()?;
        Some((key, version))
    })
}

fn digest(line: &str) -> Option<String> {
    line.split_whitespace().next().map(str::to_owned)
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

fn require_file(path: &Path) -> anyhow::Result<()> {
    if !path.is_file() {
        bail!("{} is missing", path.display());
    }
    Ok(())
}

fn read(path: &Path) -> anyhow::Result<String> {
    fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))
}

fn write(path: &Path, content: &VersionContent) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(content)?;
    fs::write(path, json).with_context(|| format!("writing {}", path.display()))
}

fn to_rfc3339<S: ser::Serializer>(
    d: &DateTime<Utc>,
    s: S,
) -> Result<S::Ok, S::Error> {
    let dt = d.to_rfc3339();
    s.serialize_str(&dt)
}
