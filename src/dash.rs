use quick_xml::{events::Event, Reader};
use std::collections::HashMap;
use std::str::FromStr;

pub struct Representation {
    pub id: i64,
    pub codecs: String,
    pub bandwidth: i64,
    pub width: Option<i64>,
    pub height: Option<i64>,
    pub frame_rate: Option<f64>,
    pub base_url: Option<String>,
}

impl Representation {
    fn get_attr<T>(e: &quick_xml::events::BytesStart, attr: &str) -> Option<T>
    where
        T: FromStr,
    {
        e.try_get_attribute(attr)
            .ok()?
            .and_then(|a| std::str::from_utf8(&a.value).ok()?.parse().ok())
    }

    pub fn from_start_event(e: quick_xml::events::BytesStart) -> Option<Self> {
        Some(Self {
            id: Self::get_attr(&e, "id")?,
            codecs: Self::get_attr(&e, "codecs")?,
            bandwidth: Self::get_attr(&e, "bandwidth")?,
            width: Self::get_attr(&e, "width"),
            height: Self::get_attr(&e, "height"),
            frame_rate: Self::get_attr(&e, "frameRate"),
            base_url: None,
        })
    }
}

pub fn parse_manifest(manifest: &str) -> Result<Vec<Representation>, quick_xml::Error> {
    let mut reader = Reader::from_str(manifest);
    reader.trim_text(true);

    let mut last_repr = None;
    let mut representations = Vec::new();
    let mut is_base_url_tag = false;

    loop {
        match reader.read_event() {
            Err(e) => return Err(e),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Representation" => last_repr = Representation::from_start_event(e),
                b"BaseURL" => is_base_url_tag = true,
                _ => (),
            },
            Ok(Event::Text(e)) => {
                if is_base_url_tag {
                    if let Some(repr) = last_repr.as_mut() {
                        repr.base_url = e.unescape().ok().map(|u| u.into_owned());
                    }
                }
            }
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"BaseURL" => is_base_url_tag = false,
                b"Representation" => {
                    if let Some(repr) = last_repr.take() {
                        representations.push(repr);
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }

    Ok(representations)
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_manifest() {
        // Read the test file
        let fname = "dash_manifest.xml";
        let mut d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("resources/test/");
        d.push(fname);
        let test_string =
            std::fs::read_to_string(d).expect(format!("Could not read {}", fname).as_str());

        // Parse the manifest
        let representations =
            super::parse_manifest(&test_string).expect("Could not parse manifest");

        assert!(representations.len() > 0, "No representations found");
    }
}
