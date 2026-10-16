//! Media type detection, based on the data's signature (via the infer crate) and the file extension.

use infer::MatcherType;

const SVG: &str = "image/svg+xml";

/// Media types for file extensions (lowercase), using the names browsers expect.
/// This is also used to normalize what infer returns, since some of its names are legacy ones
/// (e.g. application/font-woff for both WOFF and WOFF2).
fn media_type_for_extension(extension: &str) -> Option<&'static str> {
    Some(match extension {
        // Images
        "apng" => "image/apng",
        "avif" => "image/avif",
        "bmp" => "image/bmp",
        "gif" => "image/gif",
        "ico" | "cur" => "image/x-icon",
        "jpg" | "jpeg" | "jfif" | "pjpeg" | "pjp" => "image/jpeg",
        "jxl" => "image/jxl",
        "png" => "image/png",
        "svg" | "svgz" => SVG,
        "tif" | "tiff" => "image/tiff",
        "webp" => "image/webp",
        // Audio
        "aac" => "audio/aac",
        "flac" => "audio/flac",
        "m4a" => "audio/mp4",
        "mid" | "midi" => "audio/midi",
        "mp3" => "audio/mpeg",
        "oga" | "ogg" => "audio/ogg",
        "opus" => "audio/opus",
        "wav" => "audio/wav",
        "weba" => "audio/webm",
        // Video
        "avi" => "video/avi",
        "m4v" | "mp4" => "video/mp4",
        "mkv" => "video/x-matroska",
        "mov" => "video/quicktime",
        "mpeg" | "mpg" => "video/mpeg",
        "ogv" => "video/ogg",
        "webm" => "video/webm",
        // Fonts
        "eot" => "application/vnd.ms-fontobject",
        "otf" => "font/otf",
        "ttf" => "font/ttf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        // Text and documents
        "css" => "text/css",
        "csv" => "text/csv",
        "htm" | "html" => "text/html",
        "ics" => "text/calendar",
        "js" | "mjs" | "cjs" => "text/javascript",
        "json" | "map" => "application/json",
        "jsonld" => "application/ld+json",
        "md" | "markdown" => "text/markdown",
        "pdf" => "application/pdf",
        "rss" => "application/rss+xml",
        "txt" => "text/plain",
        "vtt" => "text/vtt",
        "webmanifest" => "application/manifest+json",
        "xhtml" | "xht" => "application/xhtml+xml",
        "xml" | "xsl" => "text/xml",
        // Other
        "swf" => "application/x-shockwave-flash",
        "wasm" => "application/wasm",
        "zip" => "application/zip",
        _ => return None,
    })
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
    fn svg_tag_needs_a_delimiter() {
        assert!(starts_with_svg_tag(b"<svg>"));
        assert!(starts_with_svg_tag(b"\xEF\xBB\xBF\n  <SVG width=\"1\">"));
        assert!(!starts_with_svg_tag(b"<svgfoo>"));
        assert!(!starts_with_svg_tag(b"<sv"));
    }
}
