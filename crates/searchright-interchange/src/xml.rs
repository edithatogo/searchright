//! Structural XML boundaries and text decoding for offline interchange.

use std::ops::Range;

use quick_xml::{Reader, XmlVersion, events::Event};

pub(super) struct Fragment {
    pub range: Range<usize>,
    pub error: Option<String>,
}

fn offset(reader: &Reader<&[u8]>) -> Result<usize, String> {
    usize::try_from(reader.buffer_position()).map_err(|_| "XML offset exceeds address space".into())
}

/// Recover at actual record start events, never at text in comments or CDATA.
pub(super) fn fragments(input: &str, tag: &str) -> Result<Vec<Fragment>, String> {
    let mut reader = Reader::from_str(input);
    // Envelope segmentation permits damaged records; each extracted record is
    // validated independently with strict end-name and attribute checks below.
    reader.config_mut().check_end_names = false;
    reader.config_mut().allow_unmatched_ends = true;
    let mut result = Vec::new();
    let mut active = None;
    loop {
        let before = offset(&reader)?;
        let event = reader.read_event();
        let after = offset(&reader)?;
        match event {
            Ok(Event::Start(ref start) | Event::Empty(ref start))
                if start.name().as_ref() == tag.as_bytes() =>
            {
                if let Some(previous) = active.take() {
                    result.push(Fragment {
                        range: previous..before,
                        error: Some(format!("unterminated {tag} element before next record")),
                    });
                }
                if matches!(event, Ok(Event::Empty(_))) {
                    result.push(Fragment {
                        range: before..after,
                        error: None,
                    });
                } else {
                    active = Some(before);
                }
            }
            Ok(Event::End(end)) if end.name().as_ref() == tag.as_bytes() => {
                if let Some(start) = active.take() {
                    result.push(Fragment {
                        range: start..after,
                        error: None,
                    });
                }
            }
            Ok(Event::Eof) => {
                if let Some(start) = active.take() {
                    result.push(Fragment {
                        range: start..input.len(),
                        error: Some(format!("unterminated {tag} element")),
                    });
                }
                break;
            }
            Err(error) => {
                // Lexically ambiguous input cannot safely be resynchronised by
                // searching for '<tag>' inside an unfinished attribute or CDATA.
                result.push(Fragment {
                    range: active.take().unwrap_or(before)..input.len(),
                    error: Some(format!("XML tokenisation failed: {error}")),
                });
                break;
            }
            _ => {}
        }
    }
    Ok(result)
}

fn check_characters(text: &str) -> Result<(), String> {
    if text.chars().all(|ch| {
        matches!(ch,
        '\t' | '\n' | '\r' | '\u{20}'..='\u{d7ff}' |
        '\u{e000}'..='\u{fffd}' | '\u{10000}'..='\u{10ffff}')
    }) {
        Ok(())
    } else {
        Err("XML contains a forbidden character".into())
    }
}

fn reference_text(reference: &quick_xml::events::BytesRef<'_>) -> Result<String, String> {
    let name = reference.decode().map_err(|error| error.to_string())?;
    let escaped = format!("&{name};");
    let text = quick_xml::escape::unescape(&escaped).map_err(|error| error.to_string())?;
    check_characters(&text)?;
    Ok(text.into_owned())
}

const fn name_start(ch: char) -> bool {
    matches!(ch, ':' | '_' | 'A'..='Z' | 'a'..='z' | '\u{c0}'..='\u{d6}' |
        '\u{d8}'..='\u{f6}' | '\u{f8}'..='\u{2ff}' | '\u{370}'..='\u{37d}' |
        '\u{37f}'..='\u{1fff}' | '\u{200c}'..='\u{200d}' | '\u{2070}'..='\u{218f}' |
        '\u{2c00}'..='\u{2fef}' | '\u{3001}'..='\u{d7ff}' | '\u{f900}'..='\u{fdcf}' |
        '\u{fdf0}'..='\u{fffd}' | '\u{10000}'..='\u{effff}')
}

fn check_name(bytes: &[u8]) -> Result<(), String> {
    let name = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
    let mut chars = name.chars();
    if !chars.next().is_some_and(name_start)
        || !chars.all(|ch| {
            name_start(ch)
                || matches!(ch, '-' | '.' | '0'..='9' |
            '\u{b7}' | '\u{300}'..='\u{36f}' | '\u{203f}'..='\u{2040}')
        })
    {
        return Err("invalid XML name".into());
    }
    Ok(())
}

/// No DTD expansion, external entity resolution or network access is performed.
pub(super) fn validate(xml: &str) -> Result<(), String> {
    check_characters(xml)?;
    let mut reader = Reader::from_str(xml);
    reader.config_mut().check_comments = true;
    let mut depth = 0_usize;
    let mut roots = 0_usize;
    loop {
        let event = reader.read_event().map_err(|error| error.to_string())?;
        match event {
            Event::Start(ref start) | Event::Empty(ref start) => {
                check_name(start.name().as_ref())?;
                for attribute in start.attributes() {
                    let attribute = attribute.map_err(|error| error.to_string())?;
                    check_name(attribute.key.as_ref())?;
                    if attribute.value.contains(&b'<') {
                        return Err("literal '<' in XML attribute value".into());
                    }
                    let value = attribute
                        .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
                        .map_err(|error| error.to_string())?;
                    check_characters(&value)?;
                }
                if depth == 0 {
                    roots += 1;
                }
                if matches!(event, Event::Start(_)) {
                    depth += 1;
                }
            }
            Event::End(_) => {
                depth = depth.checked_sub(1).ok_or("unmatched XML end tag")?;
            }
            Event::GeneralRef(reference) => {
                reference_text(&reference)?;
            }
            Event::DocType(_) | Event::Decl(_) => {
                return Err("DTD or XML declaration inside a record is not supported".into());
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if depth != 0 || roots != 1 {
        return Err("XML record must contain exactly one complete root element".into());
    }
    Ok(())
}

/// Select an exact path relative to the root, never fields in cited references.
pub(super) fn path<'a>(xml: &'a str, names: &[&str]) -> Vec<&'a str> {
    let mut reader = Reader::from_str(xml);
    let mut stack = Vec::new();
    let mut active = None;
    let mut result = Vec::new();
    while let Ok(before) = offset(&reader) {
        let Ok(event) = reader.read_event() else {
            break;
        };
        let Ok(after) = offset(&reader) else { break };
        match event {
            Event::Start(ref start) | Event::Empty(ref start) => {
                stack.push(start.name().as_ref().to_vec());
                let matched = stack.len() == names.len().saturating_add(1)
                    && stack
                        .iter()
                        .skip(1)
                        .zip(names)
                        .all(|(actual, name)| actual == name.as_bytes());
                if matched {
                    if matches!(event, Event::Empty(_)) {
                        result.push(&xml[before..after]);
                    } else {
                        active = Some(before);
                    }
                }
                if matches!(event, Event::Empty(_)) {
                    stack.pop();
                }
            }
            Event::End(_) => {
                if stack.len() == names.len().saturating_add(1)
                    && let Some(start) = active.take()
                {
                    result.push(&xml[start..after]);
                }
                stack.pop();
            }
            Event::Eof => break,
            _ => {}
        }
    }
    result
}

pub(super) fn attribute(xml: &str, name: &[u8]) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    let (Event::Start(start) | Event::Empty(start)) = reader.read_event().ok()? else {
        return None;
    };
    start
        .attributes()
        .filter_map(Result::ok)
        .find(|attr| attr.key.as_ref() == name)
        .and_then(|attr| {
            attr.decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
                .ok()
                .map(std::borrow::Cow::into_owned)
        })
}

/// Locate fields in an already validated record, including attributed elements.
pub(super) fn blocks<'a>(xml: &'a str, tag: &str) -> Vec<&'a str> {
    let mut reader = Reader::from_str(xml);
    let mut result = Vec::new();
    let mut active = None;
    let mut depth = 0_usize;
    while let Ok(before) = offset(&reader) {
        let Ok(event) = reader.read_event() else {
            break;
        };
        let Ok(after) = offset(&reader) else { break };
        match event {
            Event::Start(start) => {
                if active.is_some() {
                    depth += 1;
                } else if start.name().as_ref() == tag.as_bytes() {
                    active = Some(before);
                    depth = 1;
                }
            }
            Event::Empty(start) if active.is_none() && start.name().as_ref() == tag.as_bytes() => {
                if let Some(block) = xml.get(before..after) {
                    result.push(block);
                }
            }
            Event::End(_) if active.is_some() => {
                depth = depth.saturating_sub(1);
                if depth == 0
                    && let Some(start) = active.take()
                    && let Some(block) = xml.get(start..after)
                {
                    result.push(block);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    result
}

/// Decode entity references once; CDATA stays literal and markup contributes no text.
pub(super) fn text(xml: &str) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    let mut result = String::new();
    loop {
        match reader.read_event().ok()? {
            Event::Text(text) => result.push_str(&text.xml_content(XmlVersion::Implicit1_0).ok()?),
            Event::CData(text) => result.push_str(&text.xml_content(XmlVersion::Implicit1_0).ok()?),
            Event::GeneralRef(reference) => result.push_str(&reference_text(&reference).ok()?),
            Event::Eof => break,
            _ => {}
        }
    }
    let result = result.split_whitespace().collect::<Vec<_>>().join(" ");
    (!result.is_empty()).then_some(result)
}
