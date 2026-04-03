use crate::file::actions;
use crate::file::types::{CompareMethod, DesiredFileState};
use anyhow::Context;
use anyhow::bail;
use camino::Utf8Path;
use camino_tempfile::NamedUtf8TempFile;
use common::types::{ApplyOpts, ApplySummary, Changed};
use std::fs;
use util::{filter, hash, http};

pub fn run(
    path: &Utf8Path,
    desired_state: &DesiredFileState,
    compare: &CompareMethod,
    opts: &ApplyOpts,
) -> anyhow::Result<ApplySummary> {
    let mut changed = if desired_state.url_is_server {
        file_from_server(path, desired_state, compare, opts)
    } else {
        file_from_remote(path, desired_state, compare, opts)
    }?;

    if actions::ensure_metadata(path, desired_state, opts)? {
        changed = true;
    }

    apply_summary!(changed)
}

fn file_from_server(
    path: &Utf8Path,
    desired_state: &DesiredFileState,
    compare: &CompareMethod,
    opts: &ApplyOpts,
) -> anyhow::Result<Changed> {
    let url = desired_state.from_url.as_ref().context("no :from-url")?;
    let mut changed = false;

    if path.exists() {
        match compare {
            CompareMethod::Hash => {
                if hash::for_remote_file(url)? == hash::of_file(path)?.to_string() {
                    log_no_change!(path);
                } else {
                    log_updating!(path);
                    changed = true;

                    if !opts.noop {
                        http::remote_file_to_disk(url, path)?;
                    }
                }
            }
            CompareMethod::Filter(pattern) => {
                if hash::for_remote_filtered_file(url, pattern)? == hash::of_file(path)?.to_string()
                {
                    log_no_change!(path);
                } else {
                    log_updating!(path);
                    changed = true;

                    if !opts.noop {
                        http::remote_file_to_disk(url, path)?;
                    }
                }
            }
        }
    } else {
        changed = true;
        log_creating!(path);
        http::remote_file_to_disk(url, path)?;
    }

    Ok(changed)
}

fn file_from_remote(
    path: &Utf8Path,
    desired_state: &DesiredFileState,
    compare: &CompareMethod,
    opts: &ApplyOpts,
) -> anyhow::Result<Changed> {
    let url = desired_state.from_url.as_ref().context("no :from-url")?;
    let mut changed = false;
    let source = desired_state
        .from_url
        .as_ref()
        .context("no source file name")?;

    if path.exists() {
        if desired_state.only_fetch_from_url_once {
            tracing::debug!("{path} exists and :only-fetch-from-url-once is set");
        } else {
            let tmpfile = NamedUtf8TempFile::new()?;
            let temp_path = tmpfile.path();
            tracing::debug!("downloading {url} to {temp_path} for comparison");
            http::remote_file_to_disk(url, temp_path)?;

            if let Some(ref checksum) = desired_state.with_checksum
                && &hash::sha256_of_file(temp_path)? != checksum
            {
                bail!("Remote file has incorrect checksum");
            }

            match compare {
                CompareMethod::Hash => {
                    if hash::of_file(temp_path)? == hash::of_file(path)? {
                        log_no_change!(path);
                    } else {
                        changed = true;
                        log_updating!(path);

                        if !opts.noop {
                            let _bytes = fs::copy(source, path)?;
                        }
                    }
                }
                CompareMethod::Filter(pattern) => {
                    let filter = filter::FileFilter::from(pattern)?;

                    if hash::of_string(&filter.file(temp_path)?)
                        == hash::of_string(&filter.file(path)?)
                    {
                        log_no_change!(path);
                    } else {
                        changed = true;
                        log_updating!(path);

                        if !opts.noop {
                            let _bytes = fs::copy(source, path)?;
                        }
                    }
                }
            }
        }
    } else {
        changed = true;
        log_creating!(path);
        http::remote_file_to_disk(url, path)?;
    }

    Ok(changed)
}

#[cfg(test)]
mod test {
    use crate::file::ensure::GurpFileEnsure;
    use camino_tempfile_ext::prelude::*;
    use common::constants::ONE_RESOURCE_ONE_CHANGE;
    use httpmock::prelude::*;
    use pretty_assertions::assert_eq;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use tester::{defopts, janet2json, load_fixture, my_group, my_user};

    #[test]
    fn test_file_create_from_url() {
        let server = MockServer::start();

        let conf_mock = server.mock(|when, then| {
            when.method(GET).path("/sample/file");
            then.status(200)
                .header("content-type", "text/plain")
                .body(load_fixture("file/url-sample-file"));
        });

        let temp_dir = Utf8TempDir::new().unwrap();
        let temp_file = temp_dir.path().join("test-file");

        assert!(!temp_file.exists());

        let json_def = janet2json(&indoc::formatdoc! {r#"
            (file/ensure "{}"
                :from-url "{}"
                :with-checksum "9c1b427039a6c786db0277fb96e3b0851a972dcdad832441e802d8b0de936ec3"
                :mode "0640"
                :owner "{}"
                :group "{}")
            "#,
            temp_file.as_path(),
            server.url("/sample/file"),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();

        assert_eq!(ONE_RESOURCE_ONE_CHANGE, sut.apply(&defopts()).unwrap());
        assert!(temp_file.exists());
        conf_mock.assert();

        let metadata = fs::metadata(&temp_file).unwrap();

        assert_eq!(metadata.permissions().mode() & 0o7777, 0o640);
        assert_eq!(
            load_fixture("file/url-sample-file"),
            fs::read_to_string(temp_file).unwrap()
        );
    }

    #[test]
    fn test_file_create_from_url_404() {
        let server = MockServer::start();

        let conf_mock = server.mock(|when, then| {
            when.method(GET).path("/sample/file");
            then.status(404);
        });

        let json_def = janet2json(&indoc::formatdoc! {r#"
            (file/ensure "/tmp/does-not-matter"
                :from-url "{}"
                :mode "0640"
                :owner "{}"
                :group "{}")
            "#,
            server.url("/sample/file"),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        assert!(sut.apply(&defopts()).is_err());
        conf_mock.assert();
    }

    #[test]
    fn test_file_create_from_url_bad_checksum() {
        let server = MockServer::start();

        let conf_mock = server.mock(|when, then| {
            when.method(GET).path("/sample/file");
            then.status(200)
                .header("content-type", "text/plain")
                .body(load_fixture("file/url-sample-file"));
        });

        let json_def = janet2json(&indoc::formatdoc! {r#"
            (file/ensure "/tmp/does-not-matter"
                :from-url "{}"
                :with-checksum "0000000000000000000000000000000000000000000000000000000000000000"
                :mode "0640"
                :owner "{}"
                :group "{}")
            "#,
            server.url("/sample/file"),
            my_user(),
            my_group(),
        });

        let sut: GurpFileEnsure = serde_json::from_str(&json_def).unwrap();
        let err = sut.apply(&defopts()).unwrap_err();
        assert_eq!(
            err.to_string(),
            "Remote file has incorrect checksum".to_owned()
        );
        conf_mock.assert();
    }
}
