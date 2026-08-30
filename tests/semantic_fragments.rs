use mdtools::core_error::CoreError;
use mdtools::document::Document;
use mdtools::fragment::SectionFragment;
use mdtools::model::{MutationDisposition, TaskStatus};
use mdtools::patch::{
    HeadingPatchTarget, Patch, PatchOp, PatchReceipt, PreamblePatchTarget, SectionInsertionTarget,
    TableRowPatchTarget, TaskPatchTarget,
};
use mdtools::target::{TargetAddress, TargetKind, TargetSnapshot, TargetSummary};

fn section(document: &Document, heading: &str) -> TargetSnapshot {
    document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| {
            matches!(&snapshot.summary, TargetSummary::Section { heading: value, .. } if value == heading)
        })
        .unwrap()
}

fn task(document: &Document) -> TargetSnapshot {
    document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| snapshot.kind == TargetKind::Task)
        .unwrap()
}

fn semantic(markdown: &str) -> SectionFragment {
    SectionFragment::Semantic {
        markdown: markdown.into(),
    }
}

fn replace_section(
    document: &Document,
    heading: &str,
    fragment: SectionFragment,
) -> Result<mdtools::patch::PatchOutcome, CoreError> {
    Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::ReplaceSection {
            target: HeadingPatchTarget::try_from(&section(document, heading)).unwrap(),
            fragment,
        }],
    }
    .apply(document)
}

fn insert_section(
    document: &Document,
    parent: &str,
    fragment: SectionFragment,
) -> Result<mdtools::patch::PatchOutcome, CoreError> {
    Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::InsertSection {
            target: SectionInsertionTarget::try_from(&section(document, parent)).unwrap(),
            fragment,
        }],
    }
    .apply(document)
}

#[test]
fn semantic_read_is_one_relative_atx_subtree() {
    let source = "### Root\n\nbody\n\n##### Child\n\n```md\n# fenced\n```\n\n> # quoted\n\n- # listed\n\n<div>\n# html\n</div>\n\n[^1]:\n    # footnote\n";
    let document = Document::parse(source).unwrap();
    let read = document
        .resolve(&section(&document, "Root").address)
        .unwrap()
        .read_section(&document)
        .unwrap();
    assert_eq!(
        read.fragment,
        semantic("# Root\n\nbody\n\n### Child\n\n```md\n# fenced\n```\n\n> # quoted\n\n- # listed\n\n<div>\n# html\n</div>\n\n[^1]:\n    # footnote")
    );
    assert_eq!(read.markdown, source);
}

#[test]
fn semantic_boundary_variants_converge_and_crlf_is_destination_owned() {
    let expected = "# Parent\r\n\r\nbody\r\n\r\n## Child\r\n\r\ntext";
    for fragment in [
        "# Child\n\ntext",
        "\n\n# Child\n\ntext\n",
        " \r\n# Child\r\n\r\ntext\r\n\r\n",
    ] {
        let document = Document::parse("# Parent\r\n\r\nbody").unwrap();
        let outcome = insert_section(&document, "Parent", semantic(fragment)).unwrap();
        assert_eq!(outcome.document.source(), expected);
    }
}

#[test]
fn unicode_whitespace_paragraph_survives_semantic_read_and_changed_replace() {
    let source = "# A\n\nbody\n\n\u{3000}\n\n# B\n";
    let document = Document::parse(source).unwrap();
    let read = document
        .resolve(&section(&document, "A").address)
        .unwrap()
        .read_section(&document)
        .unwrap();
    assert_eq!(read.fragment, semantic("# A\n\nbody\n\n\u{3000}"));

    let SectionFragment::Semantic { markdown } = read.fragment else {
        unreachable!("section reads are semantic fragments")
    };
    let outcome =
        replace_section(&document, "A", semantic(&markdown.replace("body", "body2"))).unwrap();
    assert_eq!(
        outcome.document.source(),
        "# A\n\nbody2\n\n\u{3000}\n\n# B\n"
    );
}

#[test]
fn nonbreaking_space_before_fragment_root_is_not_a_blank_boundary() {
    let document = Document::parse("# Parent\n\nbody\n").unwrap();
    assert!(matches!(
        insert_section(&document, "Parent", semantic("\u{a0}\n\n# Child")),
        Err(CoreError::InvalidPatch(reason))
            if reason.contains("non-whitespace before its root heading")
    ));
}

#[test]
fn semantic_replacement_keeps_flush_adjacent_heading_closure() {
    let document = Document::parse("# A\nbody\n# B\nnext\n").unwrap();
    let outcome = replace_section(&document, "A", semantic("# Renamed\n\nchanged")).unwrap();
    assert_eq!(
        outcome.document.source(),
        "# Renamed\n\nchanged\n\n# B\nnext\n"
    );
    assert!(outcome
        .document
        .resolve(&section(&outcome.document, "B").address)
        .is_ok());
}

#[test]
fn setext_semantic_read_then_replace_is_byte_identical() {
    let source = "Title\n=====\n\nbody\n";
    let document = Document::parse(source).unwrap();
    let read = document
        .resolve(&section(&document, "Title").address)
        .unwrap()
        .read_section(&document)
        .unwrap();
    assert_eq!(read.fragment, semantic("# Title\n\nbody"));
    let outcome = replace_section(&document, "Title", read.fragment).unwrap();
    assert_eq!(outcome.document.source(), source);
    assert_eq!(
        outcome.receipts[0].disposition(),
        MutationDisposition::NoChange
    );
}

#[test]
fn multiline_setext_read_and_insertion_preserve_soft_break_semantics() {
    let document = Document::parse("Foo\nbar\n=====\n\nbody\n").unwrap();
    let read = document
        .resolve(&section(&document, "Foo bar").address)
        .unwrap()
        .read_section(&document)
        .unwrap();
    assert_eq!(read.fragment, semantic("# Foo bar\n\nbody"));

    let parent = Document::parse("# Parent\n\nbody").unwrap();
    let inserted = insert_section(&parent, "Parent", semantic("A\nB\n===\n\nx")).unwrap();
    assert_eq!(
        inserted.document.source(),
        "# Parent\n\nbody\n\n## A B\n\nx"
    );
}

#[test]
fn changed_setext_replacement_emits_declared_atx_source() {
    let document = Document::parse("Title\n=====\n\nbody\n").unwrap();
    let outcome = replace_section(&document, "Title", semantic("# Changed\n\nnew body")).unwrap();
    assert_eq!(outcome.document.source(), "# Changed\n\nnew body");
    assert_eq!(
        outcome.receipts[0].disposition(),
        MutationDisposition::Replaced
    );
}

#[test]
fn atx_semantic_read_then_replace_is_byte_identical() {
    let source = "## Title ##\n\nbody\n";
    let document = Document::parse(source).unwrap();
    let read = document
        .resolve(&section(&document, "Title").address)
        .unwrap()
        .read_section(&document)
        .unwrap();
    let outcome = replace_section(&document, "Title", read.fragment).unwrap();
    assert_eq!(outcome.document.source(), source);
    assert_eq!(
        outcome.receipts[0].disposition(),
        MutationDisposition::NoChange
    );
}

#[test]
fn semantic_replace_uses_the_indexed_parent_level() {
    let document = Document::parse("# Parent\n\n### Old\n\nbody\n").unwrap();
    let outcome =
        replace_section(&document, "Old", semantic("# New\n\n## Nested\n\nchanged")).unwrap();
    assert_eq!(
        outcome.document.source(),
        "# Parent\n\n## New\n\n### Nested\n\nchanged"
    );
    let PatchReceipt::ReplaceSection { outcome: receipt } = &outcome.receipts[0] else {
        panic!("replace_section receipt")
    };
    let after = match receipt {
        mdtools::patch::ReplaceSectionOutcome::NoChange { after, .. }
        | mdtools::patch::ReplaceSectionOutcome::Replaced { after, .. } => after,
    };
    assert!(outcome
        .document
        .resolve(&TargetAddress::Section {
            path: after.path.clone(),
        })
        .is_ok());
}

#[test]
fn semantic_insert_uses_each_parent_actual_level() {
    for (source, parent, expected) in [
        ("# Parent\n\nbody", "Parent", "## Child"),
        ("### Deep\n\nbody", "Deep", "#### Child"),
    ] {
        let document = Document::parse(source).unwrap();
        let outcome = insert_section(&document, parent, semantic("# Child\n\nbody")).unwrap();
        assert!(outcome.document.source().contains(expected));
        let PatchReceipt::InsertSection { outcome: receipt } = &outcome.receipts[0] else {
            panic!("insert_section receipt")
        };
        let mdtools::patch::InsertSectionOutcome::Inserted { after, .. } = receipt;
        assert!(outcome
            .document
            .resolve(&TargetAddress::Section {
                path: after.path.clone(),
            })
            .is_ok());
    }
}

#[test]
fn semantic_fragment_retains_reference_definitions() {
    let document = Document::parse("# Parent\n\nbody").unwrap();
    let fragment = "# Child\n\n[label][ref]\n\n[ref]: https://example.com";
    let outcome = insert_section(&document, "Parent", semantic(fragment)).unwrap();
    assert!(outcome
        .document
        .source()
        .contains("[ref]: https://example.com"));
    let child = section(&outcome.document, "Child");
    let read = outcome
        .document
        .resolve(&child.address)
        .unwrap()
        .read_section(&outcome.document)
        .unwrap();
    assert_eq!(read.fragment, semantic(fragment));
}

#[test]
fn move_section_relevel_preserves_the_retained_setext_refusal() {
    let document = Document::parse("# Parent\n\nbody\n\nSibling\n=======\n\nchild\n").unwrap();
    let error = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::MoveSection {
            source: HeadingPatchTarget::try_from(&section(&document, "Sibling")).unwrap(),
            destination: HeadingPatchTarget::try_from(&section(&document, "Parent")).unwrap(),
            position: mdtools::patch::SectionMovePosition::IntoAsChild,
            keep_level: false,
        }],
    }
    .apply(&document)
    .err()
    .expect("retained move API rejects setext releveling");
    assert!(matches!(
        error,
        CoreError::InvalidSelector(reason) if reason.contains("setext heading")
    ));
}

#[test]
fn literal_section_and_preamble_replacement_preserve_exact_bytes() {
    let document = Document::parse("# A\n\nbody\n").unwrap();
    let literal = "## B\r\n\r\nbody\r\n";
    let outcome = replace_section(
        &document,
        "A",
        SectionFragment::Literal {
            markdown: literal.into(),
        },
    )
    .unwrap();
    assert_eq!(outcome.document.source(), literal);

    let document = Document::parse("lead\n\n# A\n").unwrap();
    let preamble = document.resolve(&TargetAddress::Preamble).unwrap();
    let outcome = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::ReplacePreamble {
            target: PreamblePatchTarget::try_from(preamble.snapshot()).unwrap(),
            markdown: "exact\r\n".into(),
        }],
    }
    .apply(&document)
    .unwrap();
    assert_eq!(outcome.document.source(), "exact\r\n\n\n# A\n");
}

#[test]
fn invalid_semantic_fragments_fail_before_edits() {
    let document = Document::parse("# Parent\n\nbody\n").unwrap();
    for markdown in ["", "body", "lead\n\n# Child", "# One\n\n# Two"] {
        assert!(matches!(
            insert_section(&document, "Parent", semantic(markdown)),
            Err(CoreError::InvalidPatch(_))
        ));
        assert_eq!(document.source(), "# Parent\n\nbody\n");
    }
}

#[test]
fn heading_depth_overflow_is_typed_and_aborts_the_patch() {
    let document = Document::parse("##### Parent\n\nbody\n").unwrap();
    assert!(matches!(
        insert_section(&document, "Parent", semantic("# Child\n\n## Too deep")),
        Err(CoreError::HeadingDepthOverflow {
            parent_level: 5,
            relative_level: 2
        })
    ));
    assert_eq!(document.source(), "##### Parent\n\nbody\n");
}

#[test]
fn section_insertions_conflict_but_unrelated_task_edits_compose() {
    let document = Document::parse("# Parent\n\n- [ ] task\n\n# Other\n").unwrap();
    let insertion = SectionInsertionTarget::try_from(&section(&document, "Parent")).unwrap();
    let conflict = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::InsertSection {
                target: insertion.clone(),
                fragment: semantic("# One"),
            },
            PatchOp::InsertSection {
                target: insertion.clone(),
                fragment: semantic("# Two"),
            },
        ],
    };
    assert!(matches!(
        conflict.apply(&document),
        Err(CoreError::PatchInvariant(_))
    ));

    let parent = HeadingPatchTarget::try_from(&section(&document, "Parent")).unwrap();
    let conflict = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::ReplaceSection {
                target: parent,
                fragment: semantic("# Parent\n\nchanged"),
            },
            PatchOp::InsertSection {
                target: insertion.clone(),
                fragment: semantic("# Child"),
            },
        ],
    };
    assert!(matches!(
        conflict.apply(&document),
        Err(CoreError::PatchInvariant(_))
    ));

    let task = TaskPatchTarget::try_from(&task(&document)).unwrap();
    let composed = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::InsertSection {
                target: insertion,
                fragment: semantic("# Child"),
            },
            PatchOp::SetTaskStatus {
                target: task,
                status: TaskStatus::Done,
            },
        ],
    }
    .apply(&document)
    .unwrap();
    assert!(composed.document.source().contains("## Child"));
    assert!(composed.document.source().contains("- [x] task"));
}

#[test]
fn unrelated_table_edits_compose_with_section_insertion() {
    let source = "# Parent\n\nbody\n\n# Data\n\n| A | B |\n| - | - |\n| x | y |\n";
    let document = Document::parse(source).unwrap();
    let insertion = SectionInsertionTarget::try_from(&section(&document, "Parent")).unwrap();
    let row = document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| snapshot.kind == TargetKind::TableRow)
        .unwrap();
    let outcome = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::InsertSection {
                target: insertion,
                fragment: semantic("# Child"),
            },
            PatchOp::ReplaceTableRow {
                target: TableRowPatchTarget::try_from(&row).unwrap(),
                markdown: "| changed | row |".into(),
            },
        ],
    }
    .apply(&document)
    .unwrap();
    assert!(outcome.document.source().contains("## Child"));
    assert!(outcome.document.source().contains("| changed | row |"));
}

#[test]
fn two_disjoint_insertions_return_result_resolvable_receipts_in_patch_order() {
    let document = Document::parse("# A\n\na\n\n# B\n\nb\n").unwrap();
    let outcome = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::InsertSection {
                target: SectionInsertionTarget::try_from(&section(&document, "B")).unwrap(),
                fragment: semantic("# Under B"),
            },
            PatchOp::InsertSection {
                target: SectionInsertionTarget::try_from(&section(&document, "A")).unwrap(),
                fragment: semantic("# Under A"),
            },
        ],
    }
    .apply(&document)
    .unwrap();
    let headings = ["Under B", "Under A"];
    for (receipt, expected_heading) in outcome.receipts.iter().zip(headings) {
        let PatchReceipt::InsertSection { outcome: inserted } = receipt else {
            panic!("insert_section receipt")
        };
        let mdtools::patch::InsertSectionOutcome::Inserted { after, .. } = inserted;
        let resolved = outcome
            .document
            .resolve(&TargetAddress::Section {
                path: after.path.clone(),
            })
            .unwrap();
        assert!(matches!(
            &resolved.snapshot().summary,
            TargetSummary::Section { heading, .. } if heading == expected_heading
        ));
    }
}

#[test]
fn fragment_and_section_operations_are_closed_in_the_schema() {
    let schema = mdtools::protocol::protocol_schema();
    let patch = serde_json::to_string(&schema["patch"]).unwrap();
    assert!(patch.contains("SectionFragment"));
    let variants = schema["patch"]["$defs"]["SectionFragment"]["oneOf"]
        .as_array()
        .unwrap();
    for mode in ["semantic", "literal"] {
        assert_eq!(
            variants
                .iter()
                .filter(|variant| variant["properties"]["mode"]["const"] == mode)
                .count(),
            1
        );
    }
    let operations = schema["patch"]["$defs"]["PatchOp"]["oneOf"]
        .as_array()
        .unwrap();
    for operation in ["insert_section", "replace_preamble"] {
        assert_eq!(
            operations
                .iter()
                .filter(|variant| variant["properties"]["op"]["const"] == operation)
                .count(),
            1
        );
    }

    let unknown_mode = serde_json::json!({"mode":"unknown","markdown":"# A"});
    assert!(serde_json::from_value::<SectionFragment>(unknown_mode).is_err());
    let unknown_field = serde_json::json!({
        "mode":"semantic",
        "markdown":"# A",
        "unknown":true
    });
    assert!(serde_json::from_value::<SectionFragment>(unknown_field).is_err());

    let empty_heading = serde_json::json!({
        "path": [],
        "revision": "0".repeat(64),
        "guard": {
            "span": {"line_start":1,"line_end":1,"byte_start":0,"byte_end":1},
            "etag": "0".repeat(64)
        }
    });
    assert!(serde_json::from_value::<HeadingPatchTarget>(empty_heading).is_err());

    let preamble = Document::parse("lead\n\n# A\n").unwrap();
    let preamble = preamble.resolve(&TargetAddress::Preamble).unwrap();
    assert!(HeadingPatchTarget::try_from(preamble.snapshot()).is_err());
}

#[test]
fn representative_u5_checkpoint_is_reviewable() {
    let setext = Document::parse("Title\n=====\n\nbody\n").unwrap();
    let read = setext
        .resolve(&section(&setext, "Title").address)
        .unwrap()
        .read_section(&setext)
        .unwrap();
    let unchanged = replace_section(&setext, "Title", read.fragment.clone()).unwrap();
    assert_eq!(unchanged.document.source(), setext.source());

    let deep = Document::parse("##### Parent\n\nbody\n").unwrap();
    let overflow = match insert_section(&deep, "Parent", semantic("# Child\n\n## Too deep")) {
        Err(error) => error,
        Ok(_) => panic!("depth overflow unexpectedly succeeded"),
    };
    assert!(matches!(overflow, CoreError::HeadingDepthOverflow { .. }));

    let document = Document::parse("# A\n\na\n\n# B\n\nb\n").unwrap();
    let inserted = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::InsertSection {
                target: SectionInsertionTarget::try_from(&section(&document, "B")).unwrap(),
                fragment: semantic("# Under B"),
            },
            PatchOp::InsertSection {
                target: SectionInsertionTarget::try_from(&section(&document, "A")).unwrap(),
                fragment: semantic("# Under A"),
            },
        ],
    }
    .apply(&document)
    .unwrap();
    assert_eq!(inserted.receipts.len(), 2);

    let schema = mdtools::protocol::protocol_schema();
    println!(
        "canonical semantic read:\n{}",
        serde_json::to_string_pretty(&read.fragment).unwrap()
    );
    println!(
        "section fragment schema:\n{}",
        serde_json::to_string_pretty(&schema["patch"]["$defs"]["SectionFragment"]).unwrap()
    );
    println!("setext unchanged bytes:\n{}", unchanged.document.source());
    println!("depth overflow:\n{overflow}");
    println!(
        "two insertion receipts:\n{}",
        serde_json::to_string_pretty(&inserted.receipts).unwrap()
    );

    let mut impossible_receipt = serde_json::to_value(&inserted.receipts[0]).unwrap();
    impossible_receipt["outcome"]["after"]["path"] = serde_json::json!([]);
    assert!(serde_json::from_value::<PatchReceipt>(impossible_receipt).is_err());
}
