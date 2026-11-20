use lopdf::{Document, Object, Stream, dictionary};

/// Raw 8-bit gray samples with no filter, but a PNG predictor in /DecodeParms,
/// as jsPDF writes soft masks.
fn raw_mask_with_orphan_decode_parms() -> (Stream, Vec<u8>) {
    let samples: Vec<u8> = (0..216 * 170).map(|i| if i % 216 < 108 { 0 } else { 255 }).collect();
    let stream = Stream::new(
        dictionary! {
            "Type" => "XObject",
            "Subtype" => "Image",
            "Width" => 216,
            "Height" => 170,
            "ColorSpace" => "DeviceGray",
            "BitsPerComponent" => 8,
            "DecodeParms" => dictionary! {
                "Predictor" => 12, "Colors" => 1, "BitsPerComponent" => 8, "Columns" => 216,
            },
        },
        samples.clone(),
    );
    (stream, samples)
}

#[test]
fn compress_drops_decode_parms_that_described_no_filter() {
    let (mut stream, samples) = raw_mask_with_orphan_decode_parms();
    stream.compress().unwrap();

    assert_eq!(
        stream.dict.get(b"Filter").unwrap(),
        &Object::Name(b"FlateDecode".to_vec())
    );
    assert!(stream.dict.get(b"DecodeParms").is_err());
    assert_eq!(stream.decompressed_content().unwrap(), samples);
}

#[test]
fn compress_keeps_a_stream_it_does_not_compress_unchanged() {
    // Incompressible content: no filter is added, so the dictionary is left alone.
    let mut stream = Stream::new(
        dictionary! { "DecodeParms" => dictionary! { "Predictor" => 12 } },
        (0..64u32).map(|i| (i.wrapping_mul(2654435761) >> 24) as u8).collect(),
    );
    let before = stream.dict.clone();
    stream.compress().unwrap();

    assert!(stream.dict.get(b"Filter").is_err());
    assert_eq!(stream.dict, before);
}

#[test]
fn document_compress_does_not_apply_an_orphan_predictor() {
    let (stream, samples) = raw_mask_with_orphan_decode_parms();
    let mut doc = Document::with_version("1.5");
    let id = doc.add_object(stream);
    doc.compress();

    let mut saved = Vec::new();
    doc.save_to(&mut saved).unwrap();
    let reloaded = Document::load_mem(&saved).unwrap();
    let stream = reloaded.get_object(id).unwrap().as_stream().unwrap();
    assert!(stream.dict.get(b"DecodeParms").is_err());
    assert_eq!(stream.decompressed_content().unwrap(), samples);
}
