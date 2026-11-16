#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    CrLf,
}

impl LineEnding {
    pub fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::CrLf => "\r\n",
        }
    }

    pub fn detect(text: &str) -> Self {
        if text.contains("\r\n") {
            LineEnding::CrLf
        } else {
            LineEnding::Lf
        }
    }

    pub fn normalize_to_lf(text: &str) -> String {
        text.replace("\r\n", "\n")
    }

    pub fn apply(text: &str, ending: LineEnding) -> String {
        match ending {
            LineEnding::Lf => Self::normalize_to_lf(text),
            LineEnding::CrLf => {
                let lf_norm = Self::normalize_to_lf(text);
                lf_norm.replace('\n', "\r\n")
            }
        }
    }
}
