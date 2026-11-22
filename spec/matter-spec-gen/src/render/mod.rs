pub(crate) mod attributes;
pub(crate) mod bitmaps;
pub(crate) mod clusters;
pub(crate) mod commands;
pub(crate) mod context;
pub(crate) mod device_types;
pub(crate) mod enums;
pub(crate) mod events;
pub(crate) mod globals;
pub(crate) mod record;
pub mod registry;
pub(crate) mod structs;

use genco::lang::rust::Tokens;

use crate::error::Error;

/// Core trait implemented by every generated Matter specification model item.
pub trait Render {
    /// Render this model item into a `genco` Rust token stream.
    fn render_tokens(&self) -> Result<Tokens, Error>;

    /// Render this model item into formatted Rust source text.
    fn render(&self) -> Result<String, Error> {
        context::render(self.render_tokens()?)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use genco::quote;

    use super::{
        commands::single_command_tokens,
        context::{TypeContext, doc_comment, render},
        events::single_event_tokens,
    };
    use crate::model::{ClusterDef, CommandDef, EventDef, GlobalDefs};

    fn render_with_attr(raw: &str) -> String {
        render(quote! {
            $(doc_comment(raw))
            #[derive(Debug)]
            pub struct Example;
        })
        .expect("failed to render doc comment")
    }

    #[test]
    fn empty_doc_comment_emits_nothing() {
        let rendered = render_with_attr("");
        assert_eq!(rendered.trim_start(), "#[derive(Debug)]\npub struct Example;");
        assert!(!rendered.contains("///"));
    }

    #[test]
    fn single_line_doc_comment_stays_on_its_own_line() {
        let rendered = render_with_attr("Hello world.");
        assert!(
            rendered.contains("/// Hello world.\n#[derive(Debug)]"),
            "unexpected render: {rendered:?}"
        );
        assert!(
            !rendered.contains("/// Hello world.#[derive"),
            "doc comment folded into attribute: {rendered:?}"
        );
    }

    #[test]
    fn multi_line_doc_comment_preserves_paragraphs() {
        let rendered = render_with_attr("First paragraph.\n\nSecond paragraph.");
        assert!(
            rendered.contains("/// First paragraph.\n///\n/// Second paragraph.\n#[derive(Debug)]"),
            "unexpected render: {rendered:?}"
        );
        assert!(
            !rendered.contains("Second paragraph.#[derive"),
            "doc comment folded into attribute: {rendered:?}"
        );
    }

    #[test]
    fn bare_urls_wrapped_in_chevrons() {
        let rendered = render_with_attr("See https://rfc-editor.org/rfc/rfc8839#section-5.1 for details.");
        assert!(
            rendered
                .contains("/// See <https://rfc-editor.org/rfc/rfc8839#section-5.1> for details.\n#[derive(Debug)]"),
            "unexpected render: {rendered:?}"
        );
    }

    #[test]
    fn unknown_command_source_fails_fast() {
        let cluster = ClusterDef::default();
        let global_defs = GlobalDefs::default();
        let enums = BTreeMap::new();
        let bitmaps = BTreeMap::new();
        let structs = BTreeMap::new();
        let ctx = TypeContext::new(&[], &[], &[], &[], &global_defs, &enums, &bitmaps, &structs);
        let cmd = CommandDef {
            name: "Mystery".to_owned(),
            source: "sideways".to_owned(),
            ..CommandDef::default()
        };

        let err =
            single_command_tokens(&cmd, &cluster, "TestCluster", &ctx).expect_err("unknown command source must fail");
        assert!(
            err.to_string()
                .contains("Unknown command source 'sideways' on command 'Mystery'"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn event_priority_mapping_handles_known_levels() {
        let cluster = ClusterDef::default();
        let global_defs = GlobalDefs::default();
        let enums = BTreeMap::new();
        let bitmaps = BTreeMap::new();
        let structs = BTreeMap::new();
        let ctx = TypeContext::new(&[], &[], &[], &[], &global_defs, &enums, &bitmaps, &structs);

        for (input_priority, expected_token) in [
            ("critical", "EventPriority::Critical"),
            ("Critical", "EventPriority::Critical"),
            ("info", "EventPriority::Info"),
            ("Info", "EventPriority::Info"),
            ("debug", "EventPriority::Debug"),
            ("Debug", "EventPriority::Debug"),
        ] {
            let ev = EventDef {
                name: "TestEv".to_owned(),
                priority: input_priority.to_owned(),
                ..EventDef::default()
            };
            let tokens =
                single_event_tokens(&ev, &cluster, "TestCluster", &ctx).expect("valid event priority must succeed");
            let rendered = tokens.to_file_string().unwrap();
            assert!(
                rendered.contains(&format!("const PRIORITY: EventPriority = {expected_token};")),
                "failed for priority {input_priority}: {rendered}"
            );
        }
    }

    #[test]
    fn unknown_event_priority_fails_fast() {
        let cluster = ClusterDef::default();
        let global_defs = GlobalDefs::default();
        let enums = BTreeMap::new();
        let bitmaps = BTreeMap::new();
        let structs = BTreeMap::new();
        let ctx = TypeContext::new(&[], &[], &[], &[], &global_defs, &enums, &bitmaps, &structs);
        let ev = EventDef {
            name: "Mystery".to_owned(),
            priority: "sideways".to_owned(),
            ..EventDef::default()
        };

        let err =
            single_event_tokens(&ev, &cluster, "TestCluster", &ctx).expect_err("unknown event priority must fail");
        assert!(
            err.to_string()
                .contains("Unknown event priority 'sideways' on event 'Mystery'"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn keyword_field_names_are_escaped_legally() {
        use super::context::to_rust_field_name;

        assert_eq!(to_rust_field_name("self"), "self_");
        assert_eq!(to_rust_field_name("super"), "super_");
        assert_eq!(to_rust_field_name("crate"), "crate_");
        assert_eq!(to_rust_field_name("gen"), "r#gen");
        assert_eq!(to_rust_field_name("type"), "r#type");
    }

    #[test]
    fn semantic_types_resolve_to_primitive_bound_types() {
        use super::attributes::bound_primitive_type;
        use crate::model::ResolvedType;

        let temperature = ResolvedType::SemanticType {
            rust_type: "types::Temperature",
            attr_variant: "I16",
            is_copy: true,
        };
        assert_eq!(bound_primitive_type(&temperature, "types::Temperature"), "i16");
        let other = ResolvedType::SemanticType {
            rust_type: "types::IpAddress",
            attr_variant: "Bytes",
            is_copy: false,
        };
        assert_eq!(bound_primitive_type(&other, "fallback"), "fallback");
    }
}
