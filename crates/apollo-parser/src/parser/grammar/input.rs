use crate::parser::grammar::{directive, name, ty, value};
use crate::{create_err, Parser, SyntaxKind, TokenKind, S, T};

/// See: https://spec.graphql.org/June2018/#InputObjectTypeDefinition
///
/// ```txt
/// InputObjectTypeDefinition
///     Description[opt] input Name Directives[Const][opt] InputFieldsDefinition[opt]
/// ```
pub(crate) fn input_object_type_definition(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::INPUT_OBJECT_TYPE_DEFINITION);
    p.bump(SyntaxKind::input_KW);

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => {
            p.push_err(create_err!(
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected Input Object Type Definition to have a Name, got {}",
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
            ));
        }
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        input_fields_definition(p);
    }
}

/// See: https://spec.graphql.org/June2018/#InputObjectTypeExtension
///
/// ```txt
/// InputObjectTypeExtension
///     extend input Name Directives[Const][opt] InputFieldsDefinition
///     extend input Name Directives[Const]
/// ```
pub(crate) fn input_object_type_extension(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::INPUT_OBJECT_TYPE_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::input_KW);

    let mut meets_requirements = false;

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => {
            p.push_err(create_err!(
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected Input Object Type Definition to have a Name, got {}",
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
            ));
        }
    }

    if let Some(T![@]) = p.peek() {
        meets_requirements = true;
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        meets_requirements = true;
        input_fields_definition(p);
    }

    if !meets_requirements {
        p.push_err(create_err!(
            p
                .peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Expected Input Object Type Extension to have Directives or Input Fields Definition, got {}",
            p
                .peek_data()
                .unwrap_or_else(|| String::from("no further data")),
        ));
    }
}

/// See: https://spec.graphql.org/June2018/#InputFieldsDefinition
///
/// ```txt
/// InputFieldsDefinition
///     { InputValueDefinition[list] }
/// ```
pub(crate) fn input_fields_definition(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::INPUT_FIELDS_DEFINITION);
    p.bump(S!['{']);
    input_value_definition(p, false);
    if let Some(T!['}']) = p.peek() {
        p.bump(S!['}'])
    } else {
        p.push_err(create_err!(
            p.peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Expected Fields Definition to have a closing }}, got {}",
            p.peek_data()
                .unwrap_or_else(|| String::from("no further data"))
        ));
    }
}

/// See: https://spec.graphql.org/June2018/#InputValueDefinition
///
/// ```txt
/// InputValueDefinition
///     Description(opt) Name : Type DefaultValue(opt) Directives(const/opt)
/// ```
pub(crate) fn input_value_definition(p: &mut Parser, is_input: bool) {
    if let Some(TokenKind::Name) = p.peek() {
        let guard = p.start_node(SyntaxKind::INPUT_VALUE_DEFINITION);
        name::name(p);
        if let Some(T![:]) = p.peek() {
            p.bump(S![:]);
            match p.peek() {
                Some(TokenKind::Name) | Some(T!['[']) => {
                    ty::ty(p);
                    if let Some(T![=]) = p.peek() {
                        value::default_value(p);
                    }
                    if p.peek().is_some() {
                        guard.finish_node();
                        return input_value_definition(p, true);
                    }
                }
                _ => {
                    p.push_err(create_err!(
                        p.peek_data().unwrap(),
                        "Expected InputValue definition to have a Type, got {}",
                        p.peek_data().unwrap()
                    ));
                }
            }
        } else {
            p.push_err(create_err!(
                p.peek_data().unwrap(),
                "Expected InputValue definition to have a Name, got {}",
                p.peek_data().unwrap()
            ));
        }
    }
    if let Some(T![,]) = p.peek() {
        p.bump(S![,]);
        return input_value_definition(p, is_input);
    }
    // TODO @lrlna: this can be simplified a little bit, and follow the pattern of FieldDefinition
    if !is_input {
        p.push_err(create_err!(
            p.peek_data().unwrap(),
            "Expected to have an InputValue definition, got {}",
            p.peek_data().unwrap()
        ));
    }
}

#[cfg(test)]
mod test {
    use crate::parser::utils;

    #[test]
    fn it_parses_definition() {
        utils::check_ast(
            "input ExampleInputObject {
              a: String
              b: Int!
            }",
            r#"
            - DOCUMENT@0..29
                - INPUT_OBJECT_TYPE_DEFINITION@0..29
                    - input_KW@0..5 "input"
                    - NAME@5..23
                        - IDENT@5..23 "ExampleInputObject"
                    - INPUT_FIELDS_DEFINITION@23..29
                        - L_CURLY@23..24 "{"
                        - INPUT_VALUE_DEFINITION@24..26
                            - NAME@24..25
                                - IDENT@24..25 "a"
                            - COLON@25..26 ":"
                            - TYPE@26..26
                                - NAMED_TYPE@26..26
                        - INPUT_VALUE_DEFINITION@26..28
                            - NAME@26..27
                                - IDENT@26..27 "b"
                            - COLON@27..28 ":"
                            - TYPE@28..28
                                - NON_NULL_TYPE@28..28
                                    - TYPE@28..28
                                        - NAMED_TYPE@28..28
                        - R_CURLY@28..29 "}"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_name_is_missing_in_definition() {
        utils::check_ast(
            "input {
              a: String
              b: Int!
            }",
            r#"
            - DOCUMENT@0..11
                - INPUT_OBJECT_TYPE_DEFINITION@0..11
                    - input_KW@0..5 "input"
                    - INPUT_FIELDS_DEFINITION@5..11
                        - L_CURLY@5..6 "{"
                        - INPUT_VALUE_DEFINITION@6..8
                            - NAME@6..7
                                - IDENT@6..7 "a"
                            - COLON@7..8 ":"
                            - TYPE@8..8
                                - NAMED_TYPE@8..8
                        - INPUT_VALUE_DEFINITION@8..10
                            - NAME@8..9
                                - IDENT@8..9 "b"
                            - COLON@9..10 ":"
                            - TYPE@10..10
                                - NON_NULL_TYPE@10..10
                                    - TYPE@10..10
                                        - NAMED_TYPE@10..10
                        - R_CURLY@10..11 "}"
            - ERROR@0:1 "Expected Input Object Type Definition to have a Name, got {"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_enum_values_are_missing_in_definition() {
        utils::check_ast(
            "input ExampleInputObject {}",
            r#"
            - DOCUMENT@0..25
                - INPUT_OBJECT_TYPE_DEFINITION@0..25
                    - input_KW@0..5 "input"
                    - NAME@5..23
                        - IDENT@5..23 "ExampleInputObject"
                    - INPUT_FIELDS_DEFINITION@23..25
                        - L_CURLY@23..24 "{"
                        - R_CURLY@24..25 "}"
            - ERROR@0:1 "Expected to have an InputValue definition, got }"
            "#,
        )
    }

    #[test]
    fn it_parses_extension() {
        utils::check_ast(
            "extend input ExampleInputObject @skip {
              a: String
            }",
            r#"
            - DOCUMENT@0..38
                - INPUT_OBJECT_TYPE_EXTENSION@0..38
                    - extend_KW@0..6 "extend"
                    - input_KW@6..11 "input"
                    - NAME@11..29
                        - IDENT@11..29 "ExampleInputObject"
                    - DIRECTIVES@29..34
                        - DIRECTIVE@29..34
                            - AT@29..30 "@"
                            - NAME@30..34
                                - IDENT@30..34 "skip"
                    - INPUT_FIELDS_DEFINITION@34..38
                        - L_CURLY@34..35 "{"
                        - INPUT_VALUE_DEFINITION@35..37
                            - NAME@35..36
                                - IDENT@35..36 "a"
                            - COLON@36..37 ":"
                            - TYPE@37..37
                                - NAMED_TYPE@37..37
                        - R_CURLY@37..38 "}"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_name_is_missing_in_extension() {
        utils::check_ast(
            "extend input {
              a: String
            }",
            r#"
            - DOCUMENT@0..15
                - INPUT_OBJECT_TYPE_EXTENSION@0..15
                    - extend_KW@0..6 "extend"
                    - input_KW@6..11 "input"
                    - INPUT_FIELDS_DEFINITION@11..15
                        - L_CURLY@11..12 "{"
                        - INPUT_VALUE_DEFINITION@12..14
                            - NAME@12..13
                                - IDENT@12..13 "a"
                            - COLON@13..14 ":"
                            - TYPE@14..14
                                - NAMED_TYPE@14..14
                        - R_CURLY@14..15 "}"
            - ERROR@0:1 "Expected Input Object Type Definition to have a Name, got {"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_syntax_is_missing_in_extension() {
        utils::check_ast(
            "extend input ExampleInputObject",
            r#"
            - DOCUMENT@0..29
                - INPUT_OBJECT_TYPE_EXTENSION@0..29
                    - extend_KW@0..6 "extend"
                    - input_KW@6..11 "input"
                    - NAME@11..29
                        - IDENT@11..29 "ExampleInputObject"
            - ERROR@0:15 "Expected Input Object Type Extension to have Directives or Input Fields Definition, got no further data"
            "#,
        )
    }
}