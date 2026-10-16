//! Media type detection, based on the data's signature (via the infer crate) and the file extension.

use infer::MatcherType;

const SVG: &str = "image/svg+xml";

/// Media types and their file extensions (lowercase, the preferred one first),
/// using the media type names browsers expect.
/// This is also used to normalize what infer returns, since some of its names are legacy ones
/// (e.g. application/font-woff for both WOFF and WOFF2).
const MEDIA_TYPES: &[(&str, &[&str])] = &[
    // Images
    ("image/apng", &["apng"]),
    ("image/avif", &["avif"]),
    ("image/bmp", &["bmp"]),
    ("image/gif", &["gif"]),
    ("image/x-icon", &["ico", "cur"]),
    ("image/jpeg", &["jpg", "jpeg", "jfif", "pjpeg", "pjp"]),
    ("image/jxl", &["jxl"]),
    ("image/png", &["png"]),
    (SVG, &["svg", "svgz"]),
    ("image/tiff", &["tif", "tiff"]),
    ("image/webp", &["webp"]),
    // Audio
    ("audio/aac", &["aac"]),
    ("audio/flac", &["flac"]),
    ("audio/mp4", &["m4a"]),
    ("audio/midi", &["mid", "midi"]),
    ("audio/mpeg", &["mp3"]),
    ("audio/ogg", &["oga", "ogg"]),
    ("audio/opus", &["opus"]),
    ("audio/wav", &["wav"]),
    ("audio/webm", &["weba"]),
    // Video
    ("video/avi", &["avi"]),
    ("video/mp4", &["mp4", "m4v"]),
    ("video/x-matroska", &["mkv"]),
    ("video/quicktime", &["mov"]),
    ("video/mpeg", &["mpeg", "mpg"]),
    ("video/ogg", &["ogv"]),
    ("video/webm", &["webm"]),
    // Fonts
    ("application/vnd.ms-fontobject", &["eot"]),
    ("font/otf", &["otf"]),
    ("font/ttf", &["ttf"]),
    ("font/woff", &["woff"]),
    ("font/woff2", &["woff2"]),
    // Text and documents
    ("text/css", &["css"]),
    ("text/csv", &["csv"]),
    ("text/html", &["html", "htm"]),
    ("text/calendar", &["ics"]),
    ("text/javascript", &["js", "mjs", "cjs"]),
    ("application/json", &["json", "map"]),
    ("application/ld+json", &["jsonld"]),
    ("text/markdown", &["md", "markdown"]),
    ("application/pdf", &["pdf"]),
    ("application/rss+xml", &["rss"]),
    ("text/plain", &["txt"]),
    ("text/vtt", &["vtt"]),
    ("application/manifest+json", &["webmanifest"]),
    ("application/xhtml+xml", &["xhtml", "xht"]),
    ("text/xml", &["xml", "xsl"]),
    // Other
    ("application/x-shockwave-flash", &["swf"]),
    ("application/wasm", &["wasm"]),
    ("application/zip", &["zip"]),
];

/// Legacy and alternative names for media types in [`MEDIA_TYPES`]
const ALIASES: &[(&str, &str)] = &[
    ("application/javascript", "text/javascript"),
    ("application/x-javascript", "text/javascript"),
    ("text/ecmascript", "text/javascript"),
    ("application/xml", "text/xml"),
    ("image/jpg", "image/jpeg"),
    ("image/pjpeg", "image/jpeg"),
    ("image/vnd.microsoft.icon", "image/x-icon"),
    ("audio/mp3", "audio/mpeg"),
    ("audio/x-wav", "audio/wav"),
    ("audio/wave", "audio/wav"),
    ("audio/x-flac", "audio/flac"),
    ("video/x-msvideo", "video/avi"),
    ("application/font-woff", "font/woff"),
    ("application/x-font-ttf", "font/ttf"),
    ("application/x-font-otf", "font/otf"),
];

fn media_type_for_extension(extension: &str) -> Option<&'static str> {
    MEDIA_TYPES
        .iter()
        .find(|(_, extensions)| extensions.contains(&extension))
        .map(|&(media_type, _)| media_type)
}

/// Suggests a file extension (without the dot) for a media type, e.g. `"png"` for `"image/png"`.
///
/// Matching is case-insensitive, parameters are ignored, and common legacy names
/// (like `application/javascript` or `image/jpg`) are recognized too.
/// This is the counterpart of [`detect_media_type()`], meant for saving decoded data to files.
///
/// ```
/// use dataurl::extension_for_media_type;
///
/// assert_eq!(extension_for_media_type("image/png"), Some("png"));
/// assert_eq!(extension_for_media_type("Image/SVG+XML"), Some("svg"));
/// assert_eq!(extension_for_media_type("text/html; charset=utf-8"), Some("html"));
/// assert_eq!(extension_for_media_type("application/javascript"), Some("js"));
/// assert_eq!(extension_for_media_type("x/unknown"), None);
/// ```
pub fn extension_for_media_type(media_type: &str) -> Option<&'static str> {
    let essence = media_type
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let essence = ALIASES
        .iter()
        .find(|(alias, _)| *alias == essence)
        .map_or(essence.as_str(), |&(_, media_type)| media_type);
    MEDIA_TYPES
        .iter()
        .find(|(media_type, _)| *media_type == essence)
        .map(|(_, extensions)| extensions[0])
}

fn file_extension(file_name: &str) -> Option<String> {
    let (stem, extension) = file_name.rsplit_once('.')?;
    if stem.is_empty() || extension.is_empty() {
        // Dotfiles like ".htaccess" have no extension
        return None;
    }
    Some(extension.to_ascii_lowercase())
}

fn skip_leading_whitespace_and_bom(data: &[u8]) -> &[u8] {
    let data = data.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(data);
    let start = data
        .iter()
        .position(|b| !b.is_ascii_whitespace())
        .unwrap_or(data.len());
    &data[start..]
}

fn starts_with_svg_tag(data: &[u8]) -> bool {
    let data = skip_leading_whitespace_and_bom(data);
    data.len() > 4 && data[..4].eq_ignore_ascii_case(b"<svg") && !data[4].is_ascii_alphanumeric()
}

/// For XML documents, which often start with a declaration, a doctype or comments before the root element
fn contains_svg_tag(data: &[u8]) -> bool {
    data[..data.len().min(1024)]
        .windows(5)
        .any(|w| w[..4].eq_ignore_ascii_case(b"<svg") && !w[4].is_ascii_alphanumeric())
}

/// Guesses the media type of `data`, e.g. `"image/png"`.
///
/// The data's signature (its first bytes) takes precedence, which is how binary formats such as
/// images, audio, video, fonts and archives get identified, regardless of what the file is named.
/// Text formats such as CSS and JavaScript have no signature, so the extension of `file_name`
/// is used for them; HTML, XML and SVG can be recognized either way.
///
/// Returns `None` if neither the data nor the file name give a clue.
///
/// ```
/// use dataurl::detect_media_type;
///
/// assert_eq!(detect_media_type(b"\x89PNG\r\n\x1a\n", None), Some("image/png"));
/// assert_eq!(detect_media_type(b"\x89PNG\r\n\x1a\n", Some("image.jpg")), Some("image/png"));
/// assert_eq!(detect_media_type(b"body{}", Some("style.CSS")), Some("text/css"));
/// assert_eq!(detect_media_type(b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>", None), Some("image/svg+xml"));
/// assert_eq!(detect_media_type(b"just some text", None), None);
/// ```
pub fn detect_media_type(data: &[u8], file_name: Option<&str>) -> Option<&'static str> {
    let by_extension = || {
        file_name
            .and_then(file_extension)
            .and_then(|extension| media_type_for_extension(&extension))
    };

    if starts_with_svg_tag(data) {
        return Some(SVG);
    }

    match infer::get(data) {
        // Binary formats have reliable signatures
        Some(kind) if kind.matcher_type() != MatcherType::Text => {
            Some(media_type_for_extension(kind.extension()).unwrap_or(kind.mime_type()))
        }
        // Text formats (HTML, XML, shell scripts) are only recognized by how they start,
        // so the extension is more specific (e.g. an .svg or .xhtml file that starts with "<?xml")
        Some(kind) => by_extension().or_else(|| {
            if kind.mime_type() == "text/xml" && contains_svg_tag(data) {
                Some(SVG)
            } else {
                Some(media_type_for_extension(kind.extension()).unwrap_or(kind.mime_type()))
            }
        }),
        None => by_extension(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_extensions() {
        assert_eq!(file_extension("a.PNG").as_deref(), Some("png"));
        assert_eq!(file_extension("archive.tar.gz").as_deref(), Some("gz"));
        assert_eq!(file_extension(".htaccess"), None);
        assert_eq!(file_extension("README"), None);
        assert_eq!(file_extension("trailing."), None);
    }

    #[test]
    fn every_media_type_maps_back_to_its_extensions() {
        for &(media_type, extensions) in MEDIA_TYPES {
            assert_eq!(extension_for_media_type(media_type), Some(extensions[0]));
            for extension in extensions {
                assert_eq!(media_type_for_extension(extension), Some(media_type));
            }
        }
        for &(alias, media_type) in ALIASES {
            assert!(MEDIA_TYPES.iter().any(|&(m, _)| m == media_type), "{alias}");
        }
    }

    #[test]
    fn svg_tag_needs_a_delimiter() {
        assert!(starts_with_svg_tag(b"<svg>"));
        assert!(starts_with_svg_tag(b"\xEF\xBB\xBF\n  <SVG width=\"1\">"));
        assert!(!starts_with_svg_tag(b"<svgfoo>"));
        assert!(!starts_with_svg_tag(b"<sv"));
    }
}
