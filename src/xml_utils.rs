use std::io::Cursor;
use quick_xml::{Reader, Writer};
use quick_xml::events::{BytesStart, Event};

pub fn apply_settings_readonly(settings_xml_bytes: &[u8]) -> Result<Vec<u8>, quick_xml::Error> {
    let mut reader = Reader::from_reader(settings_xml_bytes);
    reader.trim_text(true);
    let mut writer = Writer::new(Cursor::new(Vec::new()));

    loop {
        match reader.read_event() {
            // Ok(Event::Start(e)) if e.name().as_ref() == b"this_tag" => {
            //
            //     // crates a new element ... alternatively we could reuse `e` by calling
            //     // `e.into_owned()`
            //     let mut elem = BytesStart::new("my_elem");
            //
            //     // collect existing attributes
            //     elem.extend_attributes(e.attributes().map(|attr| attr.unwrap()));
            //
            //     // copy existing attributes, adds a new my-key="some value" attribute
            //     elem.push_attribute(("my-key", "some value"));
            //
            //     // writes the event to the writer
            //     assert!(writer.write_event(Event::Start(elem)).is_ok());
            // },
            // Ok(Event::End(e)) if e.name().as_ref() == b"this_tag" => {
            //     assert!(writer.write_event(Event::End(BytesEnd::new("my_elem"))).is_ok());
            // },
            Ok(Event::Empty(e)) if e.name().as_ref() == b"w:documentProtection" => {
            // Ok(e) if e.name().as_ref() == b"w:documentProtection" => {
                println!("Found protection tag !!!");
                let mut elem = e.into_owned();
                elem.clear_attributes();
                // <w:documentProtection
                //     w:edit="readOnly"
                //     w:enforcement="1"
                //     w:cryptProviderType="rsaFull"
                //     w:cryptAlgorithmClass="hash"
                //     w:cryptAlgorithmType="typeAny"
                //     w:cryptAlgorithmSid="4"
                //     w:cryptSpinCount="0"
                //     w:hash="i1Ge4q/1mFrm4dpp3YwuD26Jte4="
                //     w:salt="ql69CaZ9XBh8gn1wvmwi+Q=="/>
                elem.push_attribute(("w:edit", "readOnly"));
                elem.push_attribute(("w:enforcement", "1"));
                elem.push_attribute(("w:cryptProviderType", "rsaFull"));
                elem.push_attribute(("w:cryptAlgorithmClass", "hash"));
                elem.push_attribute(("w:cryptAlgorithmType", "typeAny"));
                elem.push_attribute(("w:cryptAlgorithmSid", "4"));
                elem.push_attribute(("w:cryptSpinCount", "0"));
                elem.push_attribute(("w:hash", "i1Ge4q/1mFrm4dpp3YwuD26Jte4="));
                elem.push_attribute(("w:salt", "ql69CaZ9XBh8gn1wvmwi+Q=="));
                writer.write_event(Event::Empty(elem))?;
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
        apply_settings_readonly(settings_bytes.as_ref()).unwrap();
    }
}