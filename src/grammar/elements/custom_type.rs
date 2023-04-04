// Copyright (c) ZeroC, Inc.

use super::super::*;
use crate::slice_file::Span;
use crate::supported_encodings::SupportedEncodings;
use crate::utils::ptr_util::WeakPtr;

#[derive(Debug)]
pub struct CustomType {
    pub identifier: Identifier,
    pub parent: WeakPtr<Module>,
    pub scope: Scope,
    pub attributes: Vec<WeakPtr<Attribute>>,
    pub comment: Option<DocComment>,
    pub span: Span,
    pub(crate) supported_encodings: Option<SupportedEncodings>,
}

impl Type for CustomType {
    fn type_string(&self) -> String {
        self.identifier().to_owned()
    }

    fn fixed_wire_size(&self) -> Option<u32> {
        None
    }

    fn is_class_type(&self) -> bool {
        false
    }

    fn tag_format(&self) -> Option<TagFormat> {
        // Custom types are only supported with Slice2, which doesn't use tag formats.
        None
    }

    fn supported_encodings(&self) -> SupportedEncodings {
        self.supported_encodings.clone().unwrap()
    }
}

implement_Element_for!(CustomType, "custom type");
implement_Entity_for!(CustomType);
implement_Contained_for!(CustomType, Module);
