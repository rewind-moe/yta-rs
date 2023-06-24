use quick_xml::{events::Event, Reader};
use std::str::FromStr;

#[derive(Debug)]
pub struct Manifest {
    pub segment_duration: f64,
    pub latest_segment_number: i64,
    pub representations: Vec<Representation>,
}

#[derive(Debug)]
pub struct Representation {
    pub id: i64,
    pub codecs: String,
    pub bandwidth: i64,
    pub base_url: String,

    pub width: Option<i64>,
    pub height: Option<i64>,
    pub frame_rate: Option<f64>,
}

fn get_attr<T>(e: &quick_xml::events::BytesStart, attr: &str) -> Option<T>
where
    T: FromStr,
{
    e.try_get_attribute(attr)
        .ok()?
        .and_then(|a| std::str::from_utf8(&a.value).ok()?.parse().ok())
}

impl Representation {
    pub fn from_start_event(
        e: quick_xml::events::BytesStart,
        reader: &mut Reader<&[u8]>,
    ) -> Result<Self, quick_xml::Error> {
        let mut repr = Self {
            id: get_attr(&e, "id").ok_or(quick_xml::Error::TextNotFound)?,
            codecs: get_attr(&e, "codecs").ok_or(quick_xml::Error::TextNotFound)?,
            bandwidth: get_attr(&e, "bandwidth").ok_or(quick_xml::Error::TextNotFound)?,
            base_url: String::default(),

            width: get_attr(&e, "width"),
            height: get_attr(&e, "height"),
            frame_rate: get_attr(&e, "frameRate"),
        };

        let mut is_base_url_tag = false;
        loop {
            match reader.read_event() {
                Err(e) => return Err(e),
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"BaseURL" => is_base_url_tag = true,
                    _ => (),
                },
                Ok(Event::Text(e)) => {
                    if is_base_url_tag {
                        repr.base_url = e
                            .unescape()
                            .ok()
                            .map(|u| u.into_owned())
                            .unwrap_or_default();
                    }
                }
                Ok(Event::End(e)) => {
                    if e.name().as_ref() == b"Representation" {
                        break;
                    }
                }
                _ => (),
            }
        }

        Ok(repr)
    }

    pub fn get_url(&self, segment_number: i64) -> String {
        format!("{}sq/{}", self.base_url, segment_number)
    }
}

pub fn parse_manifest(manifest: &str) -> Result<Manifest, quick_xml::Error> {
    let mut reader = Reader::from_str(manifest);
    reader.trim_text(true);

    let mut m = Manifest {
        segment_duration: 0.0,
        latest_segment_number: 0,
        representations: Vec::new(),
    };

    loop {
        match reader.read_event() {
            Err(e) => return Err(e),
            Ok(Event::Eof) => break,
            Ok(Event::Empty(e)) => match e.name().as_ref() {
                b"S" => {
                    m.segment_duration = get_attr(&e, "d").ok_or(quick_xml::Error::TextNotFound)?;
                    m.latest_segment_number += 1;
                }
                _ => (),
            },
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"SegmentList" => {
                    m.latest_segment_number = get_attr::<i64>(&e, "startNumber")
                        .map(|n: i64| n - 1)
                        .ok_or(quick_xml::Error::TextNotFound)?;
                }
                b"Representation" => {
                    m.representations
                        .push(Representation::from_start_event(e, &mut reader)?);
                }
                _ => (),
            },
            _ => (),
        }
    }

    Ok(m)
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
        let manifest = super::parse_manifest(&test_string).expect("Could not parse manifest");

        assert!(
            manifest.representations.len() > 0,
            "No representations found"
        );
    }
}
