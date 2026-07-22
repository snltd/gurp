use crate::publisher::types::{Mirror, Origin, OriginOrMirror, Publisher};
use anyhow::Context;
use url::Url;

#[derive(PartialEq)]
enum ParseState {
    None,
    InOrigin,
    InMirror,
}

fn rows_as_key_value(raw: &str) -> Vec<(&str, &str)> {
    raw.lines()
        .filter_map(|line| {
            let mut fields = line.trim().split(": ");
            if let Some(key) = fields.next()
                && let Some(val) = fields.next()
            {
                Some((key, val))
            } else {
                None
            }
        })
        .collect()
}

/// Turns the output of `pkg publisher name` into a Publisher struct.
pub(crate) fn parse_publisher(raw: &str) -> anyhow::Result<Publisher> {
    let mut state = ParseState::None;
    let mut origins: Vec<Origin> = Vec::new();
    let mut mirrors: Vec<Mirror> = Vec::new();
    let mut in_play: Option<OriginOrMirror> = None;
    let rows = rows_as_key_value(raw);

    for (key, value) in rows {
        match key {
            "Origin URI" | "Mirror URI" => {
                if let Some(finished) = in_play.take() {
                    match state {
                        ParseState::InOrigin => origins.push(finished),
                        ParseState::InMirror => mirrors.push(finished),
                        ParseState::None => {}
                    }
                }

                in_play = Some(OriginOrMirror {
                    uri: Url::parse(value)
                        .with_context(|| format!("failed to parse uri; {value}"))?,
                    proxy: None,
                });

                state = if key == "Origin URI" {
                    ParseState::InOrigin
                } else {
                    ParseState::InMirror
                };
            }
            "Proxy" if state != ParseState::None => {
                if let Some(current) = in_play.as_mut() {
                    current.proxy = Some(
                        Url::parse(value)
                            .with_context(|| format!("failed to parse uri; {value}"))?,
                    );
                }
            }
            _ => {}
        }
    }

    if let Some(finished) = in_play {
        match state {
            ParseState::InOrigin => origins.push(finished),
            ParseState::InMirror => mirrors.push(finished),
            ParseState::None => {}
        }
    }

    Ok(Publisher {
        origins,
        mirrors: if mirrors.is_empty() {
            None
        } else {
            Some(mirrors)
        },
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_publisher() {
        let raw = indoc::indoc! {
                    r#"
                        Publisher: omnios
                        Alias:
                   Origin URI: https://pkg.omnios.org/r151056/core/
                Origin Status: Online
                        Proxy: http://10.2.0.20:3128
                      SSL Key: None
                     SSL Cert: None
                   Mirror URI: https://omnios.lan.id264.net/r151056/core/
                   Mirror Status: Online
                      SSL Key: None
                     SSL Cert: None
                   Mirror URI: https://us-west.mirror.omnios.org/r151056/core/
                   Mirror Status: Online
                        Proxy: http://localhost:3128
                      SSL Key: None
                     SSL Cert: None
                  Client UUID: e2be96f4-4496-11f1-8073-94c691ae17bc
              Catalog Updated: Wed Apr 29 18:00:41 2026
            Publisher enabled: Yes
                   Properties:
                               signature-policy = require-signatures
            "#
        };

        let expected = Publisher {
            origins: vec![Origin {
                uri: Url::parse("https://pkg.omnios.org/r151056/core/").unwrap(),
                proxy: Some(Url::parse("http://10.2.0.20:3128").unwrap()),
            }],
            mirrors: Some(vec![
                Mirror {
                    uri: Url::parse("https://omnios.lan.id264.net/r151056/core/").unwrap(),
                    proxy: None,
                },
                Mirror {
                    uri: Url::parse("https://us-west.mirror.omnios.org/r151056/core/").unwrap(),
                    proxy: Some(Url::parse("http://localhost:3128").unwrap()),
                },
            ]),
        };

        assert_eq!(parse_publisher(raw).unwrap(), expected);
    }
}
