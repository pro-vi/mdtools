use mdtools::document::Document;
use mdtools::fragment::SectionFragment;
use mdtools::patch::{
    BlockInsertionTarget, DocumentEdge, FrontmatterPatchTarget, HeadingPatchTarget, Patch, PatchOp,
    PreamblePatchTarget, RelativePosition, ReplaceBlockTarget, SectionMovePosition,
    SectionPatchTarget, TablePatchTarget, TableRowPatchTarget, TaskPatchTarget,
};
use mdtools::target::{TargetAddress, TargetKind, TargetSnapshot, TargetSummary};
use mdtools::{BlockKind, TaskStatus};

fn apply(document: &Document, operations: Vec<PatchOp>) -> Document {
    Patch {
        base_revision: document.revision().clone(),
        operations,
    }
    .apply(document)
    .unwrap()
    .document
}

fn snapshots(document: &Document, kind: TargetKind) -> Vec<TargetSnapshot> {
    document
        .map()
        .unwrap()
        .into_iter()
        .filter(|snapshot| snapshot.kind == kind)
        .collect()
}

fn block_with(document: &Document, needle: &str) -> TargetSnapshot {
    snapshots(document, TargetKind::Block)
        .into_iter()
        .find(|snapshot| matches!(&snapshot.summary, TargetSummary::Block { preview, .. } if preview.contains(needle)))
        .unwrap()
}

fn section(document: &Document, heading: &str) -> TargetSnapshot {
    snapshots(document, TargetKind::Section)
        .into_iter()
        .find(|snapshot| matches!(&snapshot.summary, TargetSummary::Section { heading: value, .. } if value == heading))
        .unwrap()
}

#[test]
fn block_delete_insert_and_move_use_canonical_evidence() {
    let document = Document::parse("one\n\ntwo\n\nthree\n").unwrap();
    let one = ReplaceBlockTarget::try_from(&block_with(&document, "one")).unwrap();
    let two = ReplaceBlockTarget::try_from(&block_with(&document, "two")).unwrap();
    let three = ReplaceBlockTarget::try_from(&block_with(&document, "three")).unwrap();

    let deleted = apply(
        &document,
        vec![PatchOp::DeleteBlock {
            target: two.clone(),
        }],
    );
    assert!(!deleted.source().contains("two"));

    let inserted = apply(
        &document,
        vec![PatchOp::InsertBlock {
            target: BlockInsertionTarget::Before {
                anchor: two.clone(),
            },
            markdown: "inserted".into(),
        }],
    );
    assert!(inserted.source().find("inserted").unwrap() < inserted.source().find("two").unwrap());

    let moved = apply(
        &document,
        vec![PatchOp::MoveBlock {
            source: three,
            destination: one,
            position: RelativePosition::Before,
        }],
    );
    assert!(moved.source().find("three").unwrap() < moved.source().find("one").unwrap());

    let edge = apply(
        &document,
        vec![PatchOp::InsertBlock {
            target: BlockInsertionTarget::DocumentEdge {
                edge: DocumentEdge::End,
                revision: document.revision().clone(),
            },
            markdown: "tail".into(),
        }],
    );
    assert!(edge.source().ends_with("tail"));
}

#[test]
fn section_replace_delete_and_move_preserve_existing_kernels() {
    let document = Document::parse("# A\n\na\n\n# B\n\nb\n\n# C\n\nc\n").unwrap();
    let a = HeadingPatchTarget::try_from(&section(&document, "A")).unwrap();
    let b = SectionPatchTarget::try_from(&section(&document, "B")).unwrap();
    let move_b = HeadingPatchTarget::try_from(&section(&document, "B")).unwrap();
    let c = HeadingPatchTarget::try_from(&section(&document, "C")).unwrap();

    let replaced = apply(
        &document,
        vec![PatchOp::ReplaceSection {
            target: a,
            fragment: SectionFragment::Semantic {
                markdown: "# A\n\nchanged".into(),
            },
        }],
    );
    assert!(replaced.source().contains("changed"));

    let deleted = apply(
        &document,
        vec![PatchOp::DeleteSection { target: b.clone() }],
    );
    assert!(!deleted.source().contains("# B"));

    let moved = apply(
        &document,
        vec![PatchOp::MoveSection {
            source: c,
            destination: move_b,
            position: SectionMovePosition::BeforeSibling,
            keep_level: true,
        }],
    );
    assert!(moved.source().find("# C").unwrap() < moved.source().find("# B").unwrap());
}

#[test]
fn task_frontmatter_and_table_operations_use_semantic_targets() {
    let source = "---\n\"a.b\": old\n---\n\n# Work\n\n- [ ] task\n\n| Name | State |\n| --- | --- |\n| A | open |\n";
    let document = Document::parse_for_frontmatter_mutation(source).unwrap();
    let task = TaskPatchTarget::try_from(&snapshots(&document, TargetKind::Task)[0]).unwrap();
    let table = snapshots(&document, TargetKind::Block)
        .into_iter()
        .find(|snapshot| {
            matches!(
                snapshot.summary,
                TargetSummary::Block {
                    kind: BlockKind::Table,
                    ..
                }
            )
        })
        .unwrap();
    let table_target = TablePatchTarget::try_from(&table).unwrap();
    let row =
        TableRowPatchTarget::try_from(&snapshots(&document, TargetKind::TableRow)[0]).unwrap();
    let field = document
        .resolve(&TargetAddress::FrontmatterField {
            path: vec!["a.b".into()],
        })
        .unwrap();
    let field = FrontmatterPatchTarget::try_from(field.snapshot()).unwrap();

    let task_changed = apply(
        &document,
        vec![PatchOp::SetTaskStatus {
            target: task,
            status: TaskStatus::Done,
        }],
    );
    assert!(task_changed.source().contains("- [x] task"));

    let frontmatter_changed = apply(
        &document,
        vec![PatchOp::SetFrontmatter {
            target: field.clone(),
            value: serde_json::json!("new"),
        }],
    );
    assert_eq!(
        frontmatter_changed
            .resolve(&TargetAddress::FrontmatterField {
                path: vec!["a.b".into()]
            })
            .unwrap()
            .read_frontmatter_field(&frontmatter_changed)
            .unwrap()
            .value,
        Some(serde_json::json!("new"))
    );
    let frontmatter_deleted = apply(
        &document,
        vec![PatchOp::DeleteFrontmatter { target: field }],
    );
    assert_eq!(
        frontmatter_deleted
            .resolve(&TargetAddress::FrontmatterField {
                path: vec!["a.b".into()]
            })
            .unwrap()
            .read_frontmatter_field(&frontmatter_deleted)
            .unwrap()
            .value,
        None
    );

    let row_replaced = apply(
        &document,
        vec![PatchOp::ReplaceTableRow {
            target: row.clone(),
            markdown: "| A | closed |".into(),
        }],
    );
    assert!(row_replaced.source().contains("| A | closed |"));
    let row_inserted = apply(
        &document,
        vec![PatchOp::InsertTableRow {
            target: table_target,
            row: 1,
            markdown: "| B | open |".into(),
        }],
    );
    assert!(row_inserted.source().contains("| B | open |"));
    let row_deleted = apply(&document, vec![PatchOp::DeleteTableRow { target: row }]);
    assert!(!row_deleted.source().contains("| A | open |"));
}

#[test]
fn frontmatter_patch_preserves_empty_key_segments() {
    let document =
        Document::parse_for_frontmatter_mutation("---\n\"\":\n  \"\": old\n---\n\nbody\n").unwrap();
    let address = TargetAddress::FrontmatterField {
        path: vec![String::new(), String::new()],
    };
    let resolved = document.resolve(&address).unwrap();
    let target = FrontmatterPatchTarget::try_from(resolved.snapshot()).unwrap();
    let changed = apply(
        &document,
        vec![PatchOp::SetFrontmatter {
            target,
            value: serde_json::json!("new"),
        }],
    );
    assert_eq!(
        changed
            .resolve(&address)
            .unwrap()
            .read_frontmatter_field(&changed)
            .unwrap()
            .value,
        Some(serde_json::json!("new"))
    );
}

#[test]
fn frontmatter_field_mutation_rejects_lossy_round_trip_and_preserves_crlf() {
    let commented = Document::parse_for_frontmatter_mutation(
        "---\n# pinned comment\naliases: [a, b]\n---\n\nbody\n",
    )
    .unwrap();
    let target = commented
        .resolve(&TargetAddress::FrontmatterField {
            path: vec!["aliases".into()],
        })
        .unwrap();
    let patch = Patch {
        base_revision: commented.revision().clone(),
        operations: vec![PatchOp::SetFrontmatter {
            target: FrontmatterPatchTarget::try_from(target.snapshot()).unwrap(),
            value: serde_json::json!(["c"]),
        }],
    };
    assert!(matches!(
        patch.apply(&commented),
        Err(mdtools::core_error::CoreError::InvalidPatch(_))
    ));

    let quoted =
        Document::parse_for_frontmatter_mutation("---\ntitle: 'hello'\nx: old\n---\n\nbody\n")
            .unwrap();
    let target = quoted
        .resolve(&TargetAddress::FrontmatterField {
            path: vec!["x".into()],
        })
        .unwrap();
    let patch = Patch {
        base_revision: quoted.revision().clone(),
        operations: vec![PatchOp::SetFrontmatter {
            target: FrontmatterPatchTarget::try_from(target.snapshot()).unwrap(),
            value: serde_json::json!("new"),
        }],
    };
    assert!(matches!(
        patch.apply(&quoted),
        Err(mdtools::core_error::CoreError::InvalidPatch(_))
    ));

    let unchanged_target = quoted
        .resolve(&TargetAddress::FrontmatterField {
            path: vec!["x".into()],
        })
        .unwrap();
    let unchanged = Patch {
        base_revision: quoted.revision().clone(),
        operations: vec![PatchOp::SetFrontmatter {
            target: FrontmatterPatchTarget::try_from(unchanged_target.snapshot()).unwrap(),
            value: serde_json::json!("old"),
        }],
    }
    .apply(&quoted)
    .unwrap();
    assert_eq!(unchanged.document.source(), quoted.source());
    assert_eq!(
        unchanged.receipts[0].disposition(),
        mdtools::MutationDisposition::NoChange
    );

    let quoted_toml = Document::parse_for_frontmatter_mutation(
        "+++\ntitle = 'hello'\nx = \"old\"\n+++\n\nbody\n",
    )
    .unwrap();
    let target = quoted_toml
        .resolve(&TargetAddress::FrontmatterField {
            path: vec!["x".into()],
        })
        .unwrap();
    let patch = Patch {
        base_revision: quoted_toml.revision().clone(),
        operations: vec![PatchOp::SetFrontmatter {
            target: FrontmatterPatchTarget::try_from(target.snapshot()).unwrap(),
            value: serde_json::json!("new"),
        }],
    };
    assert!(matches!(
        patch.apply(&quoted_toml),
        Err(mdtools::core_error::CoreError::InvalidPatch(_))
    ));

    let crlf =
        Document::parse_for_frontmatter_mutation("---\r\na: old\r\n---\r\n\r\nbody\r\n").unwrap();
    let target = crlf
        .resolve(&TargetAddress::FrontmatterField {
            path: vec!["a".into()],
        })
        .unwrap();
    let outcome = Patch {
        base_revision: crlf.revision().clone(),
        operations: vec![PatchOp::SetFrontmatter {
            target: FrontmatterPatchTarget::try_from(target.snapshot()).unwrap(),
            value: serde_json::json!("new"),
        }],
    }
    .apply(&crlf)
    .unwrap();
    assert_eq!(
        outcome.document.source(),
        "---\r\na: new\r\n---\r\n\r\nbody\r\n"
    );

    let absent = Document::parse_for_frontmatter_mutation("body\r\n").unwrap();
    let target = absent
        .resolve(&TargetAddress::FrontmatterField {
            path: vec!["a".into()],
        })
        .unwrap();
    let inserted = Patch {
        base_revision: absent.revision().clone(),
        operations: vec![PatchOp::SetFrontmatter {
            target: FrontmatterPatchTarget::try_from(target.snapshot()).unwrap(),
            value: serde_json::json!("new"),
        }],
    }
    .apply(&absent)
    .unwrap();
    assert_eq!(
        inserted.document.line_ending_style(),
        mdtools::LineEndingStyle::Crlf
    );
    assert_eq!(
        inserted.document.source(),
        "---\r\na: new\r\n---\r\n\r\nbody\r\n"
    );
}

#[test]
fn core_patch_refuses_unclaimed_frontmatter_delimiters() {
    for source in ["---\na: [\n---\n\nbody\n", "---\n\n# Title\n"] {
        let document = Document::parse(source).unwrap();
        let target = document
            .resolve(&TargetAddress::FrontmatterField {
                path: vec!["a".into()],
            })
            .unwrap();
        let patch = Patch {
            base_revision: document.revision().clone(),
            operations: vec![PatchOp::SetFrontmatter {
                target: FrontmatterPatchTarget::try_from(target.snapshot()).unwrap(),
                value: serde_json::json!("new"),
            }],
        };
        assert!(matches!(
            patch.apply(&document),
            Err(mdtools::core_error::CoreError::FrontmatterParseFailed(_))
        ));
        assert_eq!(document.source(), source);
    }
}

#[test]
fn rust_insert_block_rejects_an_empty_payload() {
    let document = Document::parse("body\n").unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::InsertBlock {
            target: BlockInsertionTarget::DocumentEdge {
                edge: DocumentEdge::End,
                revision: document.revision().clone(),
            },
            markdown: String::new(),
        }],
    };
    assert!(matches!(
        patch.apply(&document),
        Err(mdtools::core_error::CoreError::InvalidPatch(_))
    ));
}

#[test]
fn block_insertion_must_create_exactly_one_body_block() {
    let document = Document::parse("body\n").unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::InsertBlock {
            target: BlockInsertionTarget::DocumentEdge {
                edge: DocumentEdge::End,
                revision: document.revision().clone(),
            },
            markdown: "# New heading".into(),
        }],
    };
    assert!(matches!(
        patch.apply(&document),
        Err(mdtools::core_error::CoreError::InvalidPatch(_))
    ));
}

#[test]
fn block_insertion_accepts_parser_excluded_boundary_whitespace() {
    for (markdown, preview) in [
        ("two\n", "two"),
        ("two\n\n", "two"),
        (" two", "two"),
        ("  two", "two"),
        ("   two", "two"),
        ("    code", "    code"),
    ] {
        let document = Document::parse("one\n").unwrap();
        let patch = Patch {
            base_revision: document.revision().clone(),
            operations: vec![PatchOp::InsertBlock {
                target: BlockInsertionTarget::DocumentEdge {
                    edge: DocumentEdge::End,
                    revision: document.revision().clone(),
                },
                markdown: markdown.into(),
            }],
        };
        let outcome = patch.apply(&document).unwrap();
        let blocks = snapshots(&outcome.document, TargetKind::Block);
        assert!(
            blocks
                .iter()
                .any(|snapshot| matches!(&snapshot.summary, TargetSummary::Block { preview: value, .. } if value == preview)),
            "payload {markdown:?} produced {blocks:?}"
        );
    }
}

#[test]
fn block_insertion_after_and_unterminated_end_synthesizes_a_blank_boundary() {
    let mid = Document::parse("one\n\ntwo\n").unwrap();
    let one = ReplaceBlockTarget::try_from(&block_with(&mid, "one")).unwrap();
    let outcome = Patch {
        base_revision: mid.revision().clone(),
        operations: vec![PatchOp::InsertBlock {
            target: BlockInsertionTarget::After { anchor: one },
            markdown: "X".into(),
        }],
    }
    .apply(&mid)
    .unwrap();
    assert_eq!(outcome.document.source(), "one\n\nX\n\ntwo\n");

    let eof = Document::parse("one").unwrap();
    let outcome = Patch {
        base_revision: eof.revision().clone(),
        operations: vec![PatchOp::InsertBlock {
            target: BlockInsertionTarget::DocumentEdge {
                edge: DocumentEdge::End,
                revision: eof.revision().clone(),
            },
            markdown: "X".into(),
        }],
    }
    .apply(&eof)
    .unwrap();
    assert_eq!(outcome.document.source(), "one\n\nX");
}

#[test]
fn preamble_replacement_preserves_literal_boundaries() {
    for (source, markdown, expected) in [
        ("# H\n", "lead\n\n", "lead\n\n# H\n"),
        ("intro\n\n# H\n", "lead\n\n", "lead\n\n\n\n# H\n"),
    ] {
        let document = Document::parse(source).unwrap();
        let preamble = snapshots(&document, TargetKind::Preamble)
            .into_iter()
            .next()
            .unwrap();
        let outcome = Patch {
            base_revision: document.revision().clone(),
            operations: vec![PatchOp::ReplacePreamble {
                target: PreamblePatchTarget::try_from(&preamble).unwrap(),
                markdown: markdown.into(),
            }],
        }
        .apply(&document)
        .unwrap();
        assert_eq!(outcome.document.source(), expected);
        assert_eq!(
            outcome.receipts[0].disposition(),
            mdtools::MutationDisposition::Replaced
        );
    }
}

#[test]
fn heading_section_replacement_never_resolves_as_preamble() {
    let document = Document::parse("# A\n\nbody\n").unwrap();
    for markdown in ["plain", "plain\n"] {
        let patch = Patch {
            base_revision: document.revision().clone(),
            operations: vec![PatchOp::ReplaceSection {
                target: HeadingPatchTarget::try_from(&section(&document, "A")).unwrap(),
                fragment: SectionFragment::Literal {
                    markdown: markdown.into(),
                },
            }],
        };
        assert!(matches!(
            patch.apply(&document),
            Err(mdtools::core_error::CoreError::InvalidPatch(_))
        ));
    }
}

#[test]
fn block_replacement_accepts_one_space_indentation() {
    let document = Document::parse("one\n").unwrap();
    let target = ReplaceBlockTarget::try_from(&block_with(&document, "one")).unwrap();
    let outcome = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::ReplaceBlock {
            target,
            markdown: " two".into(),
        }],
    }
    .apply(&document)
    .unwrap();
    assert_eq!(outcome.document.source(), " two\n");
}

#[test]
fn block_insertion_rejects_multiple_body_blocks() {
    let document = Document::parse("one\n").unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::InsertBlock {
            target: BlockInsertionTarget::DocumentEdge {
                edge: DocumentEdge::End,
                revision: document.revision().clone(),
            },
            markdown: "two\n\nthree".into(),
        }],
    };
    let error = match patch.apply(&document) {
        Err(error) => error,
        Ok(_) => panic!("multiple-block insert unexpectedly succeeded"),
    };
    assert!(error.to_string().contains("insert_block payload"));
}

#[test]
fn block_replacement_rejects_multiple_body_blocks_with_operation_specific_error() {
    let document = Document::parse("one\n").unwrap();
    let target = ReplaceBlockTarget::try_from(&block_with(&document, "one")).unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::ReplaceBlock {
            target,
            markdown: "two\n\nthree".into(),
        }],
    };
    let error = match patch.apply(&document) {
        Err(error) => error,
        Ok(_) => panic!("multiple-block replacement unexpectedly succeeded"),
    };
    assert!(error.to_string().contains("replace_block payload"));
}

#[test]
fn block_insertion_retains_reference_definitions_owned_by_the_fragment() {
    let document = Document::parse("one\n").unwrap();
    let markdown = "[label][ref]\n\n[ref]: https://example.com\n";
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::InsertBlock {
            target: BlockInsertionTarget::DocumentEdge {
                edge: DocumentEdge::End,
                revision: document.revision().clone(),
            },
            markdown: markdown.into(),
        }],
    };

    let outcome = patch.apply(&document).unwrap();
    assert!(outcome.document.source().ends_with(markdown));
    assert_eq!(
        outcome.receipts[0].disposition(),
        mdtools::MutationDisposition::Inserted
    );
}

#[test]
fn document_start_insertion_uses_the_first_source_block_after_frontmatter() {
    let source = "---\ntitle: T\n---\n\n[^1]: note\n\nbody[^1]\n";
    let document = Document::parse_for_frontmatter(source).unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::InsertBlock {
            target: BlockInsertionTarget::DocumentEdge {
                edge: DocumentEdge::Start,
                revision: document.revision().clone(),
            },
            markdown: "start".into(),
        }],
    };

    let outcome = patch.apply(&document).unwrap();
    assert!(
        outcome.document.source().find("start").unwrap()
            < outcome.document.source().find("[^1]: note").unwrap()
    );
    assert!(
        outcome.document.source().find("---\n\n").unwrap()
            < outcome.document.source().find("start").unwrap()
    );
}

#[test]
fn section_replacement_must_preserve_one_section_target() {
    let document = Document::parse("# A\n\nbody\n").unwrap();
    let target = HeadingPatchTarget::try_from(&section(&document, "A")).unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::ReplaceSection {
            target,
            fragment: SectionFragment::Literal {
                markdown: "plain\n".into(),
            },
        }],
    };
    assert!(matches!(
        patch.apply(&document),
        Err(mdtools::core_error::CoreError::InvalidPatch(_))
    ));
}

#[test]
fn section_deletion_preserves_every_surviving_parser_block() {
    let source = "lead\n## A\n\nbody\n\nB\n===\n";
    let document = Document::parse(source).unwrap();
    let target = SectionPatchTarget::try_from(&section(&document, "A")).unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::DeleteSection { target }],
    };
    assert!(matches!(
        patch.apply(&document),
        Err(mdtools::core_error::CoreError::PatchInvariant(message))
            if message.contains("parser block")
    ));
    assert_eq!(document.source(), source);
}

#[test]
fn renamed_section_receipt_carries_the_result_address() {
    let document = Document::parse("# A\n\nbody\n").unwrap();
    let target = HeadingPatchTarget::try_from(&section(&document, "A")).unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::ReplaceSection {
            target,
            fragment: SectionFragment::Semantic {
                markdown: "# Renamed\n\nbody".into(),
            },
        }],
    };
    let outcome = patch.apply(&document).unwrap();
    let mdtools::patch::PatchReceipt::ReplaceSection {
        outcome: section_outcome,
    } = &outcome.receipts[0]
    else {
        panic!("receipt is replace_section")
    };
    let (before, after) = match section_outcome {
        mdtools::patch::ReplaceSectionOutcome::NoChange { before, after }
        | mdtools::patch::ReplaceSectionOutcome::Replaced { before, after } => (before, after),
    };
    assert_ne!(before.path, after.path);
    assert_eq!(after.revision, *outcome.document.revision());
    let target = TargetAddress::Section {
        path: after.path.clone(),
    };
    assert!(outcome.document.resolve(&target).is_ok());
}

#[test]
fn section_move_planning_preserves_strict_frontmatter_policy() {
    let source = "---\ntitle: [\n---\n# A\n\na\n\n# B\n\nb\n";
    let document = Document::parse_for_frontmatter(source).unwrap();
    let address = |text: &str| TargetAddress::Section {
        path: vec![mdtools::target::HeadingAddressSegment {
            text: text.into(),
            occurrence: 1,
        }],
    };
    let a = document.resolve(&address("A")).unwrap();
    let b = document.resolve(&address("B")).unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::MoveSection {
            source: HeadingPatchTarget::try_from(b.snapshot()).unwrap(),
            destination: HeadingPatchTarget::try_from(a.snapshot()).unwrap(),
            position: SectionMovePosition::BeforeSibling,
            keep_level: true,
        }],
    };

    let outcome = patch.apply(&document).unwrap();
    assert!(
        outcome.document.source().find("# B").unwrap()
            < outcome.document.source().find("# A").unwrap()
    );
    assert!(outcome.document.has_frontmatter());
}

#[test]
fn move_section_accepts_flush_eof_destination_without_trailing_newline() {
    let document = Document::parse("# A\n\na\n\n# B\n\nb").unwrap();
    let outcome = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::MoveSection {
            source: HeadingPatchTarget::try_from(&section(&document, "A")).unwrap(),
            destination: HeadingPatchTarget::try_from(&section(&document, "B")).unwrap(),
            position: SectionMovePosition::AfterSibling,
            keep_level: true,
        }],
    }
    .apply(&document)
    .unwrap();
    assert!(
        outcome.document.source().find("# B").unwrap()
            < outcome.document.source().find("# A").unwrap()
    );
}
