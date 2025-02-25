// Copyright (c) ZeroC, Inc.

use crate::diagnostics::{Diagnostic, Diagnostics, Error, Lint};
use crate::grammar::*;

pub fn validate_operation(operation: &Operation, diagnostics: &mut Diagnostics) {
    exception_specifications_can_only_be_used_in_slice1_mode(operation, diagnostics);
    if let Some(comment) = operation.comment() {
        validate_param_tags(comment, operation, diagnostics);
        validate_returns_tags(comment, operation, diagnostics);
        validate_throws_tags(comment, operation, diagnostics);
    }
}

fn exception_specifications_can_only_be_used_in_slice1_mode(operation: &Operation, diagnostics: &mut Diagnostics) {
    if operation.encoding != CompilationMode::Slice1 && !operation.exception_specification.is_empty() {
        // Create a span that covers the the entire exception specification.
        let mut span = operation.exception_specification.first().unwrap().span().clone();
        span.end = operation.exception_specification.last().unwrap().span().end;

        Diagnostic::new(Error::ExceptionSpecificationNotSupported)
            .set_span(&span)
            .set_scope(operation.parser_scoped_identifier())
            .push_into(diagnostics);
    }
}

fn validate_param_tags(comment: &DocComment, operation: &Operation, diagnostics: &mut Diagnostics) {
    let parameters: Vec<_> = operation.parameters().iter().map(|p| p.identifier()).collect();

    for param_tag in &comment.params {
        let tag_identifier = param_tag.identifier.value.as_str();
        if !parameters.contains(&tag_identifier) {
            Diagnostic::new(Lint::IncorrectDocComment {
                message: format!(
                    "comment has a 'param' tag for '{tag_identifier}', but operation '{}' has no parameter with that name",
                    operation.identifier(),
                ),
            })
            .set_span(param_tag.span())
            .set_scope(operation.parser_scoped_identifier())
            .push_into(diagnostics);
        }
    }
}

fn validate_returns_tags(comment: &DocComment, operation: &Operation, diagnostics: &mut Diagnostics) {
    let returns_tags = &comment.returns;
    match operation.return_members().as_slice() {
        // If the operation doesn't return anything, but its doc comment has 'returns' tags, report an error.
        [] => validate_returns_tags_for_operation_with_no_return_type(returns_tags, operation, diagnostics),

        // If the operation returns a single type, ensure that its 'returns' tag doesn't have an identifier.
        [_] => validate_returns_tags_for_operation_with_single_return(returns_tags, operation, diagnostics),

        // If the operation returns a tuple, ensure its returns tags use identifiers matching the tuple's.
        tuple => validate_returns_tags_for_operation_with_return_tuple(returns_tags, operation, tuple, diagnostics),
    }
}

fn validate_returns_tags_for_operation_with_no_return_type(
    returns_tags: &[ReturnsTag],
    operation: &Operation,
    diagnostics: &mut Diagnostics,
) {
    for returns_tag in returns_tags {
        Diagnostic::new(Lint::IncorrectDocComment {
            message: format!(
                "comment has a 'returns' tag, but operation '{}' does not return anything",
                operation.identifier(),
            ),
        })
        .set_span(&(returns_tag.span() + returns_tag.message.span()))
        .set_scope(operation.parser_scoped_identifier())
        .push_into(diagnostics);
    }
}

fn validate_returns_tags_for_operation_with_single_return(
    returns_tags: &[ReturnsTag],
    operation: &Operation,
    diagnostics: &mut Diagnostics,
) {
    for returns_tag in returns_tags {
        if let Some(tag_identifier) = &returns_tag.identifier {
            Diagnostic::new(Lint::IncorrectDocComment {
                message: format!(
                    "comment has a 'returns' tag for '{}', but operation '{}' doesn't return anything with that name",
                    &tag_identifier.value,
                    operation.identifier(),
                ),
            })
            .set_span(returns_tag.span())
            .set_scope(operation.parser_scoped_identifier())
            .add_note(
                format!("operation '{}' returns a single unnamed type", operation.identifier()),
                Some(operation.span()),
            )
            .add_note("try removing the identifier from your comment: \"@returns: ...\"", None)
            .push_into(diagnostics);
        }
    }
}

fn validate_returns_tags_for_operation_with_return_tuple(
    returns_tags: &[ReturnsTag],
    operation: &Operation,
    return_tuple: &[&Parameter],
    diagnostics: &mut Diagnostics,
) {
    let return_members: Vec<_> = return_tuple.iter().map(|p| p.identifier()).collect();

    for returns_tag in returns_tags {
        if let Some(tag_identifier) = &returns_tag.identifier {
            let tag_identifier = tag_identifier.value.as_str();
            if !return_members.contains(&tag_identifier) {
                Diagnostic::new(Lint::IncorrectDocComment {
                    message: format!(
                        "comment has a 'returns' tag for '{tag_identifier}', but operation '{}' doesn't return anything with that name",
                        operation.identifier(),
                    ),
                })
                .set_span(returns_tag.span())
                .set_scope(operation.parser_scoped_identifier())
                .push_into(diagnostics);
            }
        }
    }
}

fn validate_throws_tags(comment: &DocComment, operation: &Operation, diagnostics: &mut Diagnostics) {
    let throws_tags = &comment.throws;
    if operation.exception_specification.is_empty() {
        // If the operation doesn't throw, but its doc comment has 'throws' tags, report an error.
        validate_throws_tags_for_operation_with_no_throws_clause(throws_tags, operation, diagnostics);
    } else {
        // If the operation can throw exceptions, ensure that its 'throws' tags agree with them.
        let thrown_exceptions = &operation.exception_specification;
        validate_throws_tags_for_operation_with_throws_clause(throws_tags, operation, thrown_exceptions, diagnostics);
    }
}

fn validate_throws_tags_for_operation_with_no_throws_clause(
    throws_tags: &[ThrowsTag],
    operation: &Operation,
    diagnostics: &mut Diagnostics,
) {
    for throws_tag in throws_tags {
        Diagnostic::new(Lint::IncorrectDocComment {
            message: format!(
                "comment has a 'throws' tag, but operation '{}' does not throw anything",
                operation.identifier(),
            ),
        })
        .set_span(&(throws_tag.span() + throws_tag.message.span()))
        .set_scope(operation.parser_scoped_identifier())
        .push_into(diagnostics);
    }
}

fn validate_throws_tags_for_operation_with_throws_clause(
    throws_tags: &[ThrowsTag],
    operation: &Operation,
    exception_types: &[TypeRef<Exception>],
    diagnostics: &mut Diagnostics,
) {
    for throws_tag in throws_tags {
        if let Ok(documented_exception) = throws_tag.thrown_type() {
            let is_correct = exception_types.iter().any(|thrown_exception| {
                is_documented_exception_compatible(thrown_exception.definition(), documented_exception)
            });

            if !is_correct {
                Diagnostic::new(Lint::IncorrectDocComment {
                    message: format!(
                        "comment has a 'throws' tag for '{}', but operation '{}' doesn't throw this exception",
                        documented_exception.identifier(),
                        operation.identifier(),
                    ),
                })
                .set_span(throws_tag.span())
                .set_scope(operation.parser_scoped_identifier())
                .push_into(diagnostics);
            }
        }
    }
}

/// Returns true if `documented_exception` is the same as, or derives from `thrown_exception`.
fn is_documented_exception_compatible(thrown_exception: &Exception, documented_exception: &Exception) -> bool {
    if std::ptr::eq(thrown_exception, documented_exception) {
        true
    } else if let Some(base_exception) = documented_exception.base_exception() {
        is_documented_exception_compatible(thrown_exception, base_exception)
    } else {
        false
    }
}
