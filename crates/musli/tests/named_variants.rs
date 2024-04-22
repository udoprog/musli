use musli::{Decode, Encode};

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(name_all = "PascalCase")]
enum PascalCase {
    VariantName,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(name_all = "camelCase")]
enum CamelCase {
    VariantName,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(name_all = "snake_case")]
enum SnakeCase {
    VariantName,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(name_all = "SCREAMING_SNAKE_CASE")]
enum ScreamingSnakeCase {
    VariantName,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(name_all = "kebab-case")]
enum KebabCase {
    VariantName,
}

#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[musli(name_all = "SCREAMING-KEBAB-CASE")]
enum ScreamingKebabCase {
    VariantName,
}

#[test]
fn test_name_all() {
    musli::rt!(full, PascalCase::VariantName, json = r#""VariantName""#,);

    musli::rt!(full, CamelCase::VariantName, json = r#""variantName""#,);

    musli::rt!(full, SnakeCase::VariantName, json = r#""variant_name""#,);

    musli::rt!(
        full,
        ScreamingSnakeCase::VariantName,
        json = r#""VARIANT_NAME""#,
    );

    musli::rt!(full, KebabCase::VariantName, json = r#""variant-name""#,);

    musli::rt!(
        full,
        ScreamingKebabCase::VariantName,
        json = r#""VARIANT-NAME""#,
    );
}
