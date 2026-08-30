use crate::model::LineEndingStyle;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceEdit {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) replacement: String,
}

pub(crate) fn normalize_line_endings(content: &str, style: LineEndingStyle) -> String {
    match style {
        LineEndingStyle::Lf => content.replace("\r\n", "\n"),
        LineEndingStyle::Crlf => content.replace("\r\n", "\n").replace('\n', "\r\n"),
        LineEndingStyle::Mixed => content.to_string(),
    }
}

pub(crate) fn strip_one_trailing_newline(mut content: String) -> String {
    if content.ends_with("\r\n") {
        content.truncate(content.len() - 2);
    } else if content.ends_with('\n') {
        content.truncate(content.len() - 1);
    }
    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_ending_normalization_preserves_each_style_contract() {
        assert_eq!(
            normalize_line_endings("one\r\ntwo\n", LineEndingStyle::Lf),
            "one\ntwo\n"
        );
        assert_eq!(
            normalize_line_endings("one\r\ntwo\n", LineEndingStyle::Crlf),
            "one\r\ntwo\r\n"
        );
        assert_eq!(
            normalize_line_endings("one\r\ntwo\n", LineEndingStyle::Mixed),
            "one\r\ntwo\n"
        );
    }

    #[test]
    fn trailing_newline_removal_strips_exactly_one_line_ending() {
        assert_eq!(strip_one_trailing_newline("row\r\n".into()), "row");
        assert_eq!(strip_one_trailing_newline("row\n\n".into()), "row\n");
        assert_eq!(strip_one_trailing_newline("row".into()), "row");
    }
}
