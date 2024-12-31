use core::fmt;
use core::mem::take;

use crate::expander::{NameMethod, UnsizedMethod};

#[derive(Default, Debug, Clone, Copy)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum NameAll {
    /// Fields are named by index.
    #[default]
    Index,
    /// Fields are named by their original name.
    Name,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

impl NameAll {
    pub(crate) const ALL: &'static [Self] = &[
        Self::Index,
        Self::Name,
        Self::PascalCase,
        Self::CamelCase,
        Self::SnakeCase,
        Self::ScreamingSnakeCase,
        Self::KebabCase,
        Self::ScreamingKebabCase,
    ];

    pub(crate) fn ty(&self) -> syn::Type {
        match self {
            NameAll::Index => syn::parse_quote! { usize },
            _ => syn::parse_quote! { str },
        }
    }

    pub(crate) fn name_method(&self) -> NameMethod {
        match self {
            NameAll::Index => NameMethod::Sized,
            _ => NameMethod::Unsized(UnsizedMethod::Default),
        }
    }

    pub(crate) fn parse(input: &str) -> Option<Self> {
        match input {
            "index" => Some(Self::Index),
            "name" => Some(Self::Name),
            "PascalCase" => Some(Self::PascalCase),
            "camelCase" => Some(Self::CamelCase),
            "snake_case" => Some(Self::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Some(Self::ScreamingSnakeCase),
            "kebab-case" => Some(Self::KebabCase),
            "SCREAMING-KEBAB-CASE" => Some(Self::ScreamingKebabCase),
            _ => None,
        }
    }

    /// Apply the given rename to the input string.
    pub(crate) fn apply(&self, input: &str) -> Option<String> {
        let feed: fn(output: &mut String, open: bool, count: usize, c: char) = match self {
            Self::Index => return None,
            Self::Name => return Some(input.to_string()),
            Self::PascalCase => |output, open, _, c| {
                if open {
                    output.extend(c.to_uppercase());
                } else {
                    output.extend(c.to_lowercase());
                }
            },
            Self::CamelCase => |output, open, count, c| {
                if open && count > 0 {
                    output.extend(c.to_uppercase());
                } else {
                    output.extend(c.to_lowercase());
                }
            },
            Self::SnakeCase => |output, _, _, c| {
                output.extend(c.to_lowercase());
            },
            Self::ScreamingSnakeCase => |output, _, _, c| {
                output.extend(c.to_uppercase());
            },
            Self::KebabCase => |output, _, _, c| {
                output.extend(c.to_lowercase());
            },
            Self::ScreamingKebabCase => |output, _, _, c| {
                output.extend(c.to_uppercase());
            },
        };

        let prefix = match self {
            Self::SnakeCase => Some('_'),
            Self::ScreamingSnakeCase => Some('_'),
            Self::KebabCase => Some('-'),
            Self::ScreamingKebabCase => Some('-'),
            _ => None,
        };

        let mut output = String::new();
        let mut count = 0;
        let mut group = true;

        for c in input.chars() {
            if matches!(c, '_') {
                if !group {
                    count += 1;
                }

                group = true;
                continue;
            }

            if char::is_uppercase(c) && !group {
                group = true;
                count += 1;
            }

            let group = take(&mut group);

            if group && count > 0 {
                output.extend(prefix);
            }

            feed(&mut output, group, count, c);
        }

        Some(output)
    }
}

impl fmt::Display for NameAll {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Index => write!(f, "index"),
            Self::Name => write!(f, "name"),
            Self::PascalCase => write!(f, "PascalCase"),
            Self::CamelCase => write!(f, "camelCase"),
            Self::SnakeCase => write!(f, "snake_case"),
            Self::ScreamingSnakeCase => write!(f, "SCREAMING_SNAKE_CASE"),
            Self::KebabCase => write!(f, "kebab-case"),
            Self::ScreamingKebabCase => write!(f, "SCREAMING-KEBAB-CASE"),
        }
    }
}

#[test]
fn test_rename() {
    #[track_caller]
    fn test(input: &str, rename: &str, expected: &str) {
        let rename = NameAll::parse(rename).unwrap();
        assert_eq!(rename.apply(input).unwrap(), expected);
    }

    test("hello_world", "PascalCase", "HelloWorld");
    test("__hello__world__", "PascalCase", "HelloWorld");
    test("hello_world", "camelCase", "helloWorld");
    test("__hello__world__", "camelCase", "helloWorld");
    test("hello_world", "snake_case", "hello_world");
    test("__hello__world__", "snake_case", "hello_world");
    test("hello_world", "SCREAMING_SNAKE_CASE", "HELLO_WORLD");
    test("__hello__world__", "SCREAMING_SNAKE_CASE", "HELLO_WORLD");
    test("hello_world", "kebab-case", "hello-world");
    test("__hello__world__", "kebab-case", "hello-world");
    test("hello_world", "SCREAMING-KEBAB-CASE", "HELLO-WORLD");
    test("__hello__world__", "SCREAMING-KEBAB-CASE", "HELLO-WORLD");

    test("HelloWorld", "PascalCase", "HelloWorld");
    test("__Hello__World__", "PascalCase", "HelloWorld");
    test("HelloWorld", "camelCase", "helloWorld");
    test("__Hello__World__", "camelCase", "helloWorld");
    test("HelloWorld", "snake_case", "hello_world");
    test("__Hello__World__", "snake_case", "hello_world");
    test("HelloWorld", "SCREAMING_SNAKE_CASE", "HELLO_WORLD");
    test("__Hello__World__", "SCREAMING_SNAKE_CASE", "HELLO_WORLD");
    test("HelloWorld", "kebab-case", "hello-world");
    test("__Hello__World__", "kebab-case", "hello-world");
    test("HelloWorld", "SCREAMING-KEBAB-CASE", "HELLO-WORLD");
    test("__Hello__World__", "SCREAMING-KEBAB-CASE", "HELLO-WORLD");
}
