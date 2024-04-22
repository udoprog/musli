use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(name_all = "PascalCase")]
struct PascalCase {
    field_name: i32,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(name_all = "camelCase")]
struct CamelCase {
    field_name: i32,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(name_all = "snake_case")]
struct SnakeCase {
    field_name: i32,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(name_all = "SCREAMING_SNAKE_CASE")]
struct ScreamingSnakeCase {
    field_name: i32,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(name_all = "kebab-case")]
struct KebabCase {
    field_name: i32,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(name_all = "SCREAMING-KEBAB-CASE")]
struct ScreamingKebabCase {
    field_name: i32,
}

#[test]
fn test_name_all() {
    musli::rt!(
        full,
        PascalCase { field_name: 42 },
        json = r#"{"FieldName":42}"#,
    );

    musli::rt!(
        full,
        CamelCase { field_name: 42 },
        json = r#"{"fieldName":42}"#,
    );

    musli::rt!(
        full,
        SnakeCase { field_name: 42 },
        json = r#"{"field_name":42}"#,
    );

    musli::rt!(
        full,
        ScreamingSnakeCase { field_name: 42 },
        json = r#"{"FIELD_NAME":42}"#,
    );

    musli::rt!(
        full,
        KebabCase { field_name: 42 },
        json = r#"{"field-name":42}"#,
    );

    musli::rt!(
        full,
        ScreamingKebabCase { field_name: 42 },
        json = r#"{"FIELD-NAME":42}"#,
    );
}
