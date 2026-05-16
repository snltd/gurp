use crate::publisher::types::{Mirror, Origin, OriginOrMirror, Publisher};
#[derive(PartialEq)]
enum ParseState {
    None,
    InOrigin,
    InMirror,
}

/// Turns the output of `pkg publisher name` into a Publisher struct
pub(crate) fn parse_publisher(raw: &str) -> Publisher {
    let mut state = ParseState::None;
    let mut origins: Vec<Origin> = Vec::new();
    let mut mirrors: Vec<Mirror> = Vec::new();

    let rows: Vec<_> = raw
        .lines()
        .filter_map(|line| {
            let mut fields = line.trim().split(": ");
            if let Some(key) = fields.next()
                && let Some(val) = fields.next()
            {
                Some([key, val])
            } else {
                None
            }
        })
        .collect();

    let mut in_play = OriginOrMirror::default();

    for [key, value] in rows {
        match key {
            "Origin URI" | "Mirror URI" => {
                if state == ParseState::InOrigin {
                    origins.push(in_play);
                } else if state == ParseState::InMirror {
                    mirrors.push(in_play);
                }

                in_play = OriginOrMirror {
                    uri: value.to_owned(),
                    ..Default::default()
                };

                state = if key == "Origin URI" {
                    ParseState::InOrigin
                } else {
                    ParseState::InMirror
                };
            }
            "Proxy" if state != ParseState::None => in_play.proxy = Some(value.to_owned()),
            _ => {}
        }
    }

    if state == ParseState::InOrigin {
        origins.push(in_play);
    } else if state == ParseState::InMirror {
        mirrors.push(in_play);
    }

    Publisher {
        origins,
        mirrors: if mirrors.is_empty() {
            None
        } else {
            Some(mirrors)
        },
    }
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
                uri: "https://pkg.omnios.org/r151056/core/".to_owned(),
                proxy: Some("http://10.2.0.20:3128".to_owned()),
            }],
            mirrors: Some(vec![
                Mirror {
                    uri: "https://omnios.lan.id264.net/r151056/core/".to_owned(),
                    proxy: None,
                },
                Mirror {
                    uri: "https://us-west.mirror.omnios.org/r151056/core/".to_owned(),
                    proxy: None,
                },
            ]),
        };

        assert_eq!(parse_publisher(raw), expected);
    }
}
