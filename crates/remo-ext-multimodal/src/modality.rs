//! Modality types for representing multimodal content.
//!
//! Defines [`ModalityType`] for classifying media kinds, [`MediaSource`]
//! for identifying where media data originates, [`MediaDescriptor`] for
//! describing individual media items, and [`MultimodalContent`] for
//! grouping multiple content items together.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The type of modality for a content item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ModalityType {
    /// Plain text content.
    Text,
    /// Image content (raster or vector graphics).
    Image,
    /// Audio content (speech, music, sound effects).
    Audio,
    /// Video content (motion picture with or without audio).
    Video,
    /// Generic file content (binary or structured data).
    File,
}

impl std::fmt::Display for ModalityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModalityType::Text => write!(f, "text"),
            ModalityType::Image => write!(f, "image"),
            ModalityType::Audio => write!(f, "audio"),
            ModalityType::Video => write!(f, "video"),
            ModalityType::File => write!(f, "file"),
        }
    }
}

/// Where media data originates from.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MediaSource {
    /// Inline base64-encoded data.
    Base64(String),
    /// Remote URL pointing to the media resource.
    Url(String),
    /// Local filesystem path.
    FilePath(String),
}

/// Describes a single piece of media content with its modality, MIME type,
/// and source location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MediaDescriptor {
    /// The modality classification of this media item.
    pub modality: ModalityType,
    /// MIME type string (e.g. `"image/png"`, `"text/plain"`).
    pub mime_type: String,
    /// Where the media data can be accessed.
    pub source: MediaSource,
}

/// A collection of multimodal content items.
///
/// Wraps a vector of [`MediaDescriptor`]s to represent a message or payload
/// that may contain multiple types of media.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MultimodalContent {
    /// Ordered list of content items.
    pub items: Vec<MediaDescriptor>,
}

impl MultimodalContent {
    /// Create an empty multimodal content collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a collection with a single item.
    pub fn single(descriptor: MediaDescriptor) -> Self {
        Self {
            items: vec![descriptor],
        }
    }

    /// Add a content item to the collection.
    pub fn push(&mut self, descriptor: MediaDescriptor) {
        self.items.push(descriptor);
    }

    /// Returns `true` if the collection contains no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of items in the collection.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Filter items by modality type.
    pub fn filter_by_modality(&self, modality: ModalityType) -> Vec<&MediaDescriptor> {
        self.items
            .iter()
            .filter(|item| item.modality == modality)
            .collect()
    }

    /// Returns `true` if any item has the given modality.
    pub fn has_modality(&self, modality: ModalityType) -> bool {
        self.items.iter().any(|item| item.modality == modality)
    }
}

impl FromIterator<MediaDescriptor> for MultimodalContent {
    fn from_iter<I: IntoIterator<Item = MediaDescriptor>>(iter: I) -> Self {
        Self {
            items: iter.into_iter().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modality_type_display() {
        assert_eq!(ModalityType::Text.to_string(), "text");
        assert_eq!(ModalityType::Image.to_string(), "image");
        assert_eq!(ModalityType::Audio.to_string(), "audio");
        assert_eq!(ModalityType::Video.to_string(), "video");
        assert_eq!(ModalityType::File.to_string(), "file");
    }

    #[test]
    fn modality_type_serde_roundtrip() {
        for modality in [
            ModalityType::Text,
            ModalityType::Image,
            ModalityType::Audio,
            ModalityType::Video,
            ModalityType::File,
        ] {
            let json = serde_json::to_string(&modality).unwrap();
            let parsed: ModalityType = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, modality);
        }
    }

    #[test]
    fn multimodal_content_basics() {
        let mut content = MultimodalContent::new();
        assert!(content.is_empty());
        assert_eq!(content.len(), 0);

        content.push(MediaDescriptor {
            modality: ModalityType::Text,
            mime_type: "text/plain".to_string(),
            source: MediaSource::FilePath("/tmp/doc.txt".to_string()),
        });

        assert!(!content.is_empty());
        assert_eq!(content.len(), 1);
        assert!(content.has_modality(ModalityType::Text));
        assert!(!content.has_modality(ModalityType::Image));
    }

    #[test]
    fn multimodal_content_filter_by_modality() {
        let content = MultimodalContent {
            items: vec![
                MediaDescriptor {
                    modality: ModalityType::Text,
                    mime_type: "text/plain".to_string(),
                    source: MediaSource::FilePath("/tmp/a.txt".to_string()),
                },
                MediaDescriptor {
                    modality: ModalityType::Image,
                    mime_type: "image/png".to_string(),
                    source: MediaSource::Url("https://example.com/img.png".to_string()),
                },
                MediaDescriptor {
                    modality: ModalityType::Text,
                    mime_type: "text/markdown".to_string(),
                    source: MediaSource::Base64("aGVsbG8=".to_string()),
                },
            ],
        };

        let texts = content.filter_by_modality(ModalityType::Text);
        assert_eq!(texts.len(), 2);

        let images = content.filter_by_modality(ModalityType::Image);
        assert_eq!(images.len(), 1);
    }

    #[test]
    fn media_source_serde_roundtrip() {
        let sources = vec![
            MediaSource::Base64("aGVsbG8=".to_string()),
            MediaSource::Url("https://example.com/file.bin".to_string()),
            MediaSource::FilePath("/home/user/doc.pdf".to_string()),
        ];
        for source in sources {
            let json = serde_json::to_string(&source).unwrap();
            let parsed: MediaSource = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, source);
        }
    }

    #[test]
    fn from_iterator() {
        let items = vec![
            MediaDescriptor {
                modality: ModalityType::Text,
                mime_type: "text/plain".to_string(),
                source: MediaSource::FilePath("a.txt".to_string()),
            },
            MediaDescriptor {
                modality: ModalityType::Image,
                mime_type: "image/jpeg".to_string(),
                source: MediaSource::Url("https://example.com/photo.jpg".to_string()),
            },
        ];
        let content: MultimodalContent = items.into_iter().collect();
        assert_eq!(content.len(), 2);
    }
}
