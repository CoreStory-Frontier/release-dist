use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{anychar, one_of};
use nom::combinator::{eof, map_res, peek, recognize, success, value};
use nom::multi::many_till;
use nom::sequence::preceded;
use nom::{IResult, Parser};
use semver::Version;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct Platform {
    pub(crate) signature: String,
    pub(crate) url: url::Url,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub(crate) enum PlatformKey {
    #[serde(rename = "darwin-aarch64")]
    DarwinArm,
    #[serde(rename = "linux-x86_64")]
    LinuxX64,
    #[serde(rename = "windows-x86_64")]
    WindowsX64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum Product {
    Cli,
    Desktop,
}

pub(crate) struct ReleaseAsset {
    pub(crate) product: Product,
    pub(crate) version: Option<Version>,
    pub(crate) platform: PlatformKey,
}

impl ReleaseAsset {
    pub(crate) fn parse(i: &str) -> IResult<&str, Self> {
        let versioned = (
            preceded(sep, version).map(Some),
            preceded(sep, platform),
            extension,
        );
        let unversioned = (
            success(None),
            success(PlatformKey::DarwinArm),
            tag(".app.tar.gz"),
        );
        (product, alt((versioned, unversioned)), eof)
            .map(|(product, (version, platform, _), _)| Self {
                product,
                version,
                platform,
            })
            .parse(i)
    }
}

fn product(i: &str) -> IResult<&str, Product> {
    alt((
        value(Product::Cli, tag("corestory-cli")),
        value(Product::Desktop, tag("CoreStory")),
    ))
    .parse(i)
}

fn sep(i: &str) -> IResult<&str, char> {
    one_of("-_").parse(i)
}

fn version(i: &str) -> IResult<&str, Version> {
    map_res(
        recognize(many_till(anychar, peek((sep, platform)))),
        Version::parse,
    )
    .parse(i)
}

fn platform(i: &str) -> IResult<&str, PlatformKey> {
    alt((
        value(PlatformKey::DarwinArm, tag("aarch64-apple-darwin")),
        value(PlatformKey::LinuxX64, tag("x86_64-unknown-linux-gnu")),
        value(PlatformKey::LinuxX64, tag("x86_64-unknown-linux-musl")),
        value(PlatformKey::WindowsX64, tag("x86_64-pc-windows-gnu")),
        value(PlatformKey::DarwinArm, tag("aarch64")),
        value(PlatformKey::LinuxX64, tag("amd64")),
    ))
    .parse(i)
}

fn extension(i: &str) -> IResult<&str, &str> {
    alt((tag(".app.tar.gz"), tag(".tar.gz"), tag(".deb"))).parse(i)
}
