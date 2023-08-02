// Copyright (c) ZeroC, Inc.

use super::*;

#[derive(Debug)]
pub struct Deprecated {
    pub reason: Option<String>,
}

impl Deprecated {
    pub fn parse_from(Unparsed { directive, args }: &Unparsed, span: &Span, diagnostics: &mut Diagnostics) -> Self {
        debug_assert_eq!(directive, Self::directive());

        check_that_at_most_one_argument_was_provided(args, Self::directive(), span, diagnostics);

        let reason = args.first().cloned();
        Deprecated { reason }
    }

    pub fn validate_on(&self, applied_on: Attributables, span: &Span, diagnostics: &mut Diagnostics) {
        match applied_on {
            Attributables::Module(_) | Attributables::TypeRef(_) | Attributables::SliceFile(_) => {
                report_unexpected_attribute(self, span, None, diagnostics);
            }
            Attributables::Parameter(_) => {
                let note = "parameters cannot be individually deprecated";
                report_unexpected_attribute(self, span, Some(note), diagnostics);
            }
            _ => {}
        }
    }
}

implement_attribute_kind_for!(Deprecated, "deprecated", false);
