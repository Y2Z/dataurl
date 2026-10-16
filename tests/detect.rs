use dataurl::detect_media_type;

/// Pads a file header with zeros, since some signatures need a minimum length
/// (e.g. WebM needs more than 256 bytes to be told apart from other Matroska files)
fn header(start: &[u8]) -> Vec<u8> {
    let mut data = start.to_vec();
    data.resize(512.max(start.len()), 0);
    data
}

#[test]
fn binary_formats_by_signature() {
    for (data, expected) in [
        (header(b"\x89PNG\r\n\x1a\n"), "image/png"),
        (header(b"\xFF\xD8\xFF\xE0"), "image/jpeg"),
        (header(b"GIF87a"), "image/gif"),
        (header(b"GIF89a"), "image/gif"),
        (header(b"RIFF\x24\x00\x00\x00WEBPVP8 "), "image/webp"),
        (header(b"\x00\x00\x01\x00\x01\x00"), "image/x-icon"),
        (header(b"BM\x36\x00\x0C\x00\x00\x00"), "image/bmp"),
        (header(b"%PDF-1.7"), "application/pdf"),
        (header(b"ID3\x04\x00"), "audio/mpeg"),
        (header(b"fLaC\x00\x00\x00\x22"), "audio/flac"),
        (header(b"OggS\x00\x02"), "audio/ogg"),
        (header(b"RIFF\x24\x00\x00\x00WAVEfmt "), "audio/wav"),
        (header(b"RIFF\x24\x00\x00\x00AVI LIST"), "video/avi"),
        (
            header(b"\x00\x00\x00\x18ftypisom\x00\x00\x02\x00"),
            "video/mp4",
        ),
        (
            header(b"\x00\x00\x00\x14ftypqt  \x00\x00\x00\x00"),
            "video/quicktime",
        ),
        (header(b"\x00\x00\x01\xBA"), "video/mpeg"),
        (header(b"wOFF\x00\x01\x00\x00"), "font/woff"),
        (header(b"wOF2\x00\x01\x00\x00"), "font/woff2"),
        (header(b"\x00\x01\x00\x00\x00"), "font/ttf"),
        (header(b"OTTO\x00"), "font/otf"),
        (header(b"\x00asm\x01\x00\x00\x00"), "application/wasm"),
        (header(b"PK\x03\x04"), "application/zip"),
    ] {
        assert_eq!(detect_media_type(&data, None), Some(expected), "{expected}");
    }
}

#[test]
fn webm() {
    // EBML header with "webm" as the document type
    let data = b"\x1A\x45\xDF\xA3\x9F\x42\x86\x81\x01\x42\xF7\x81\x01\x42\xF2\x81\x04\x42\xF3\x81\x08\x42\x82\x84webm\x42\x87\x81\x04";
    assert_eq!(detect_media_type(&header(data), None), Some("video/webm"));
}

#[test]
fn signature_beats_extension() {
    let png = header(b"\x89PNG\r\n\x1a\n");
    assert_eq!(
        detect_media_type(&png, Some("photo.jpg")),
        Some("image/png")
    );
    assert_eq!(
        detect_media_type(&png, Some("style.css")),
        Some("image/png")
    );
}

#[test]
fn text_formats_by_extension() {
    for (file_name, expected) in [
        ("style.css", "text/css"),
        ("STYLE.CSS", "text/css"),
        ("app.js", "text/javascript"),
        ("module.mjs", "text/javascript"),
        ("data.json", "application/json"),
        ("notes.txt", "text/plain"),
        ("subtitles.vtt", "text/vtt"),
        ("site.webmanifest", "application/manifest+json"),
        ("font.woff2", "font/woff2"),
        ("image.SVG", "image/svg+xml"),
    ] {
        assert_eq!(
            detect_media_type(b"", Some(file_name)),
            Some(expected),
            "{file_name}"
        );
    }
}

#[test]
fn html() {
    let html = b"<!DOCTYPE html><html><head></head><body></body></html>";
    assert_eq!(detect_media_type(html, None), Some("text/html"));
    assert_eq!(detect_media_type(html, Some("page.htm")), Some("text/html"));
    assert_eq!(
        detect_media_type(html, Some("page.xhtml")),
        Some("application/xhtml+xml")
    );
}

#[test]
fn svg() {
    assert_eq!(detect_media_type(b"<svg ", None), Some("image/svg+xml"));
    assert_eq!(
        detect_media_type(
            b"\n  <svg xmlns=\"http://www.w3.org/2000/svg\"></svg>",
            None
        ),
        Some("image/svg+xml")
    );

    let with_declaration =
        b"<?xml version=\"1.0\"?>\n<!-- comment -->\n<svg xmlns=\"http://www.w3.org/2000/svg\"/>";
    assert_eq!(
        detect_media_type(with_declaration, None),
        Some("image/svg+xml")
    );
    assert_eq!(
        detect_media_type(b"<?xml version=\"1.0\"?>", Some("drawing.svg")),
        Some("image/svg+xml")
    );
}

#[test]
fn xml() {
    let xml = b"<?xml version=\"1.0\"?><feed></feed>";
    assert_eq!(detect_media_type(xml, None), Some("text/xml"));
    assert_eq!(
        detect_media_type(xml, Some("feed.rss")),
        Some("application/rss+xml")
    );
    // An element merely starting with "svg" isn't an SVG
    assert_eq!(
        detect_media_type(b"<?xml version=\"1.0\"?><svgish/>", None),
        Some("text/xml")
    );
}

#[test]
fn script_with_shebang_uses_extension() {
    assert_eq!(
        detect_media_type(b"#!/usr/bin/env node\nconsole.log(1);", Some("cli.js")),
        Some("text/javascript")
    );
}

#[test]
fn unknown() {
    assert_eq!(detect_media_type(b"", None), None);
    assert_eq!(detect_media_type(b"just some text", None), None);
    assert_eq!(
        detect_media_type(b"abcdef0123456789", Some("file.unknown")),
        None
    );
    assert_eq!(detect_media_type(b"x", Some("README")), None);
    assert_eq!(detect_media_type(b"x", Some(".css")), None);
}
