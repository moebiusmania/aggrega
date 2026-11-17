//! Checking whether a newer Aggrega is out, from the project's GitHub
//! Releases. It's the one request that isn't about the user's own reading,
//! so it only happens when they ask for it ("Check now") or turn on the daily
//! check: `App` decides when; this module only asks and compares.
//!
//! The request is a plain GET of `releases/latest` (which skips drafts and
//! pre-releases), carrying nothing but a `User-Agent: aggrega/<version>`.

use std::cmp::Ordering;
use std::fmt;

use crate::fetch;

/// Where the latest release is described.
pub const LATEST_URL: &str = "https://api.github.com/repos/moebiusmania/aggrega/releases/latest";

/// Release pages the notice may open: anything else in the answer is ignored.
const RELEASE_PAGES: &str = "https://github.com/moebiusmania/aggrega/releases/";

/// The version this build was stamped with.
pub const CURRENT: &str = env!("CARGO_PKG_VERSION");

/// Builds made for a package manager (`AGGREGA_NO_UPDATE_CHECK` set at build
/// time) leave updates to it, and don't offer the check at all.
pub const AVAILABLE: bool = option_env!("AGGREGA_NO_UPDATE_CHECK").is_none();

/// How long the daily check waits after the last one.
pub const INTERVAL_SECS: i64 = 24 * 60 * 60;

/// The answer is far smaller; this only stops a runaway body.
const MAX_BYTES: u64 = 1024 * 1024;

/// Why an update check failed.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("couldn't download the latest release")]
    Fetch(#[from] fetch::Error),
    #[error("the answer isn't JSON")]
    Malformed(#[source] serde_json::Error),
    /// JSON, but not a release (say, GitHub's error message).
    #[error("the answer doesn't describe a release")]
    NotARelease,
    #[error("the latest release's tag “{tag}” isn't a version number")]
    BadTag { tag: String },
}

/// A `major.minor.patch` version, optionally a pre-release (`1.0.0-rc1`,
/// or the `0.0.0-dev` of untagged builds), which sorts before the release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: Option<String>,
}

impl Version {
    /// Parses `1.2.3`, `v1.2.3` or `1.2.3-rc1`. Build metadata (`+…`) is
    /// ignored, as semver says.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        let s = s.strip_prefix('v').unwrap_or(s);
        let s = s.split_once('+').map_or(s, |(v, _)| v);
        let (core, pre) = match s.split_once('-') {
            Some((core, pre)) if !pre.is_empty() => (core, Some(pre.to_string())),
            Some(_) => return None,
            None => (s, None),
        };
        let mut parts = core.split('.').map(|p| {
            // Digits only: `u64::from_str` would also take a leading `+`.
            (!p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
                .then(|| p.parse().ok())
                .flatten()
        });
        let (Some(Some(major)), Some(Some(minor)), Some(Some(patch)), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return None;
        };
        Some(Self {
            major,
            minor,
            patch,
            pre,
        })
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.major, self.minor, self.patch)
            .cmp(&(other.major, other.minor, other.patch))
            .then_with(|| match (&self.pre, &other.pre) {
                (None, None) => Ordering::Equal,
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                // Good enough for comparing pre-releases of the same version.
                (Some(a), Some(b)) => a.cmp(b),
            })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.pre {
            write!(f, "-{pre}")?;
        }
        Ok(())
    }
}

/// A published release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub version: Version,
    /// Its page on GitHub: release notes and downloads.
    pub url: String,
}

impl Release {
    /// The release page for `version`, used when the answer's own link
    /// isn't one of the project's release pages.
    fn page(version: &Version) -> String {
        format!("{RELEASE_PAGES}tag/v{version}")
    }

    /// The release as the `latest_release` setting keeps it, so the notice
    /// survives a restart without asking again.
    pub fn encode(&self) -> String {
        format!("{} {}", self.version, self.url)
    }

    pub fn decode(saved: &str) -> Option<Self> {
        let (version, url) = saved.split_once(' ')?;
        let version = Version::parse(version)?;
        let url = if url.starts_with(RELEASE_PAGES) {
            url.to_string()
        } else {
            Self::page(&version)
        };
        Some(Self { version, url })
    }
}

/// What a check found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    UpToDate,
    Available(Release),
}

/// Reads a `releases/latest` answer. `Ok(None)` for a draft or a pre-release,
/// which never count as an update.
pub fn parse_release(json: &[u8]) -> Result<Option<Release>, Error> {
    let value: serde_json::Value = serde_json::from_slice(json).map_err(Error::Malformed)?;
    let flag = |key: &str| value.get(key).and_then(|v| v.as_bool()).unwrap_or(false);
    if flag("draft") || flag("prerelease") {
        return Ok(None);
    }
    let Some(tag) = value.get("tag_name").and_then(|v| v.as_str()) else {
        return Err(Error::NotARelease);
    };
    let version = Version::parse(tag).ok_or_else(|| Error::BadTag {
        tag: tag.to_string(),
    })?;
    if version.pre.is_some() {
        return Ok(None);
    }
    let url = match value.get("html_url").and_then(|v| v.as_str()) {
        Some(url) if url.starts_with(RELEASE_PAGES) => url.to_string(),
        _ => Release::page(&version),
    };
    Ok(Some(Release { version, url }))
}

/// Whether `latest` is newer than `current`. A build whose own version doesn't
/// parse is never told to update.
pub fn compare(current: &str, latest: Option<Release>) -> Status {
    match (Version::parse(current), latest) {
        (Some(current), Some(release)) if release.version > current => Status::Available(release),
        _ => Status::UpToDate,
    }
}

/// Asks `url` (`LATEST_URL`, except in tests) for the latest release.
/// Blocking: run it on a worker thread.
pub fn check(agent: &ureq::Agent, url: &str) -> Result<Status, Error> {
    let latest = match fetch::fetch_release(agent, url, MAX_BYTES)? {
        Some(body) => parse_release(&body)?,
        // No release published yet.
        None => None,
    };
    Ok(compare(CURRENT, latest))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LATEST: &[u8] = include_bytes!("../tests/fixtures/release-latest.json");

    fn v(s: &str) -> Version {
        Version::parse(s).unwrap_or_else(|| panic!("{s} parses"))
    }

    #[test]
    fn parses_versions() {
        assert_eq!(
            v("v1.2.3"),
            Version {
                major: 1,
                minor: 2,
                patch: 3,
                pre: None
            }
        );
        assert_eq!(v("1.2.3"), v("v1.2.3"));
        assert_eq!(v(" 1.2.3+build.5 "), v("1.2.3"));
        assert_eq!(v("1.0.0-rc1").pre.as_deref(), Some("rc1"));
        assert_eq!(v("0.0.0-dev").to_string(), "0.0.0-dev");
        for bad in [
            "", "v", "1", "1.2", "1.2.3.4", "1.2.x", "1..3", "+1.2.3", "1.+2.3", "1.2.3-",
            "latest", "v 1.2.3", "１.2.3",
        ] {
            assert_eq!(Version::parse(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn compares_versions() {
        assert!(v("1.2.4") > v("1.2.3"));
        assert!(v("1.10.0") > v("1.9.9"), "numerically, not as text");
        assert!(v("2.0.0") > v("1.99.99"));
        assert!(
            v("1.0.0") > v("1.0.0-rc1"),
            "a release beats its pre-release"
        );
        assert!(v("1.0.0-rc2") > v("1.0.0-rc1"));
        assert!(v("0.0.1") > v("0.0.0-dev"));
        assert_eq!(v("1.2.3").cmp(&v("v1.2.3+meta")), Ordering::Equal);
    }

    #[test]
    fn reads_a_saved_latest_release() {
        let release = parse_release(LATEST).unwrap().unwrap();
        assert_eq!(release.version, v("1.2.0"));
        assert_eq!(
            release.url,
            "https://github.com/moebiusmania/aggrega/releases/tag/v1.2.0"
        );
    }

    #[test]
    fn pre_releases_and_drafts_are_not_updates() {
        let json = |extra: &str| format!(r#"{{"tag_name": "v9.0.0"{extra}}}"#);
        assert!(parse_release(json("").as_bytes()).unwrap().is_some());
        for extra in [r#", "prerelease": true"#, r#", "draft": true"#] {
            assert_eq!(parse_release(json(extra).as_bytes()).unwrap(), None);
        }
        let rc = br#"{"tag_name": "v9.0.0-rc1", "prerelease": false}"#;
        assert_eq!(parse_release(rc).unwrap(), None, "a pre-release tag");
    }

    #[test]
    fn malformed_answers_are_errors() {
        assert!(matches!(
            parse_release(b"<html>rate limited</html>"),
            Err(Error::Malformed(_))
        ));
        assert!(matches!(
            parse_release(br#"{"message": "Not Found"}"#),
            Err(Error::NotARelease)
        ));
        assert!(matches!(
            parse_release(br#"{"tag_name": "nightly"}"#),
            Err(Error::BadTag { tag }) if tag == "nightly"
        ));
    }

    #[test]
    fn saved_releases_round_trip() {
        let release = parse_release(LATEST).unwrap().unwrap();
        assert_eq!(Release::decode(&release.encode()), Some(release));
        assert_eq!(Release::decode(""), None);
        assert_eq!(Release::decode("garbage"), None);
        assert_eq!(
            Release::decode("1.3.0 https://evil.example/").unwrap().url,
            "https://github.com/moebiusmania/aggrega/releases/tag/v1.3.0"
        );
    }

    #[test]
    fn only_opens_the_projects_release_pages() {
        let json = br#"{"tag_name": "v1.3.0", "html_url": "https://evil.example/aggrega"}"#;
        assert_eq!(
            parse_release(json).unwrap().unwrap().url,
            "https://github.com/moebiusmania/aggrega/releases/tag/v1.3.0"
        );
    }

    #[test]
    fn newer_releases_are_updates() {
        let latest = |tag: &str| {
            Some(Release {
                version: v(tag),
                url: String::new(),
            })
        };
        assert!(matches!(
            compare("1.2.0", latest("1.3.0")),
            Status::Available(r) if r.version == v("1.3.0")
        ));
        assert_eq!(compare("1.3.0", latest("1.3.0")), Status::UpToDate);
        assert_eq!(compare("1.4.0", latest("1.3.0")), Status::UpToDate);
        assert_eq!(compare("1.2.0", None), Status::UpToDate);
        assert_eq!(compare("not a version", latest("1.3.0")), Status::UpToDate);
        // Untagged builds are older than any release.
        assert!(matches!(
            compare("0.0.0-dev", latest("0.1.0")),
            Status::Available(_)
        ));
    }

    #[test]
    fn checks_against_a_server() {
        let url = fetch::serve("application/json", LATEST.to_vec(), 1);
        let status = check(&fetch::agent(), &url).unwrap();
        // Dev builds are 0.0.0-dev; release builds carry their tag.
        if v(CURRENT) < v("1.2.0") {
            assert!(matches!(status, Status::Available(r) if r.version == v("1.2.0")));
        } else {
            assert_eq!(status, Status::UpToDate);
        }
    }

    #[test]
    fn an_unreachable_server_is_a_fetch_error() {
        let err = check(&fetch::agent(), "http://127.0.0.1:9/latest").unwrap_err();
        assert!(
            matches!(err, Error::Fetch(fetch::Error::Unreachable { .. })),
            "{err}"
        );
    }
}
