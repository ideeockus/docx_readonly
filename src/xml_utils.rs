use std::io::Cursor;
use quick_xml::{Reader, Writer};
use quick_xml::events::{BytesStart, Event};

pub fn apply_settings_readonly(settings_xml_bytes: impl AsRef<[u8]>) -> Result<Vec<u8>, quick_xml::Error> {
    let mut reader = Reader::from_reader(settings_xml_bytes.as_ref());
    reader.trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    loop {
        match reader.read_event() {
            Ok(Event::Empty(e)) if e.name().as_ref() == b"w:documentProtection" => {
                println!("Found protection tag !!!");
            }

            Ok(Event::Start(e)) if e.name().as_ref() == b"w:settings" => {
                let mut elem = e.into_owned();
                writer.write_event(Event::Start(elem))?;

                let mut protection_elem = BytesStart::new("w:documentProtection");
                protection_elem.clear_attributes();
                protection_elem.push_attribute(("w:edit", "readOnly"));
                protection_elem.push_attribute(("w:enforcement", "1"));
                protection_elem.push_attribute(("w:cryptProviderType", "rsaFull"));
                protection_elem.push_attribute(("w:cryptAlgorithmClass", "hash"));
                protection_elem.push_attribute(("w:cryptAlgorithmType", "typeAny"));
                protection_elem.push_attribute(("w:cryptAlgorithmSid", "4"));
                protection_elem.push_attribute(("w:cryptSpinCount", "0"));
                protection_elem.push_attribute(("w:hash", "i1Ge4q/1mFrm4dpp3YwuD26Jte4="));
                protection_elem.push_attribute(("w:salt", "ql69CaZ9XBh8gn1wvmwi+Q=="));
                writer.write_event(Event::Empty(protection_elem))?;
            },
            Ok(Event::Eof) => break,
            // we can either move or borrow the event to write, depending on your use-case
            Ok(e) => {
                println!("{:?}", e);
                writer.write_event(e)?;
            },
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
        }
    }

    println!("Result is\n\n {}", std::str::from_utf8(writer.clone().into_inner().into_inner().as_ref()).unwrap());
    Ok(writer.into_inner().into_inner())
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;
    use crate::xml_utils::apply_settings_readonly;

    #[test]
    fn apply_settings_readonly_test() {
        let readonly_settings_path = Path::new("test_artifacts/settings_readonly_pass_123.xml");
        let mut file = File::open(readonly_settings_path).unwrap();
        let mut settings_bytes = Vec::<u8>::new();
        file.read_to_end(&mut settings_bytes).unwrap();
        apply_settings_readonly(settings_bytes).unwrap();
    }
}