use liveletters_utils::text::{NonBlankError, require_non_blank};

#[test]
fn accepts_non_blank_string_without_trimming_it() {
    let value = "  Alice  ";

    assert_eq!(require_non_blank(value, "nickname").unwrap(), value);
}

#[test]
fn rejects_empty_string_with_field_name() {
    let err = require_non_blank("", "nickname").unwrap_err();

    assert_eq!(err, NonBlankError { field: "nickname" });
}

#[test]
fn rejects_whitespace_only_string() {
    let err = require_non_blank(" \n\t", "body").unwrap_err();

    assert_eq!(err.field, "body");
}
