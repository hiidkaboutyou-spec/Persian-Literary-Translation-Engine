use serde::{Deserialize, Serialize};

pub const BOOK_IR_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookIr {
    pub version: u32,
    pub blocks: Vec<BookIrBlock>,
}

impl BookIr {
    pub fn new(blocks: Vec<BookIrBlock>) -> Self {
        Self {
            version: BOOK_IR_VERSION,
            blocks,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookIrBlock {
    /// Stable identity inherited from the canonical manuscript/source block.
    pub id: String,
    pub runs: Vec<BookIrRun>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BookIrRun {
    Text { text: String },
    Protected {
        text: String,
        protected_kind: ProtectedSpanKind,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedSpanKind {
    Url,
    Email,
    Isbn,
    Identifier,
    Code,
    FilePath,
    ExplicitLatin,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_zwnj_byte_for_byte() {
        let text = "می\u{200c}روم و برمی\u{200c}گردم";
        let ir = BookIr::new(vec![BookIrBlock {
            id: "paragraph-1".to_owned(),
            runs: vec![BookIrRun::Text {
                text: text.to_owned(),
            }],
        }]);

        let encoded = serde_json::to_vec(&ir).expect("Book IR should serialize");
        let decoded: BookIr = serde_json::from_slice(&encoded).expect("Book IR should deserialize");

        assert_eq!(decoded, ir);
        let BookIrRun::Text { text: decoded_text } = &decoded.blocks[0].runs[0] else {
            panic!("expected text run");
        };
        assert_eq!(decoded_text.as_bytes(), text.as_bytes());
    }

    #[test]
    fn protected_runs_round_trip_without_rewriting_text() {
        let url = "https://example.com/a?x=1&y=2";
        let ir = BookIr::new(vec![BookIrBlock {
            id: "paragraph-2".to_owned(),
            runs: vec![
                BookIrRun::Text {
                    text: "لینک: ".to_owned(),
                },
                BookIrRun::Protected {
                    text: url.to_owned(),
                    protected_kind: ProtectedSpanKind::Url,
                },
            ],
        }]);

        let encoded = serde_json::to_string(&ir).expect("Book IR should serialize");
        let decoded: BookIr = serde_json::from_str(&encoded).expect("Book IR should deserialize");

        assert_eq!(decoded.version, BOOK_IR_VERSION);
        assert_eq!(decoded, ir);
        let BookIrRun::Protected { text, .. } = &decoded.blocks[0].runs[1] else {
            panic!("expected protected run");
        };
        assert_eq!(text, url);
    }
}
