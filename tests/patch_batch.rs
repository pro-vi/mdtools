use mdtools::document::Document;
use mdtools::patch::{
    BlockInsertionTarget, FrontmatterPatchTarget, HeadingPatchTarget, Patch, PatchOp,
    ReplaceBlockTarget, SectionMovePosition, TableRowPatchTarget, TaskPatchTarget,
};
use mdtools::target::{TargetAddress, TargetKind, TargetSnapshot, TargetSummary};
use mdtools::{BlockKind, TaskStatus};

fn block_with(document: &Document, needle: &str) -> TargetSnapshot {
    document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| {
            matches!(&snapshot.summary, TargetSummary::Block { preview, .. } if preview.contains(needle))
        })
        .unwrap()
}

#[test]
fn disjoint_patch_applies_once() {
    let source = "---\nstatus: old\n---\n\n# Work\n\n- [ ] task\n\n| Name | State |\n| --- | --- |\n| A | open |\n";
    let document = Document::parse_for_frontmatter_mutation(source).unwrap();
    let map = document.map().unwrap();
    let task = TaskPatchTarget::try_from(map.iter().find(|s| s.kind == TargetKind::Task).unwrap())
        .unwrap();
    let row =
        TableRowPatchTarget::try_from(map.iter().find(|s| s.kind == TargetKind::TableRow).unwrap())
            .unwrap();
    let field = document
        .resolve(&TargetAddress::FrontmatterField {
            path: vec!["status".into()],
        })
        .unwrap();
    let field = FrontmatterPatchTarget::try_from(field.snapshot()).unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::SetFrontmatter {
                target: field,
                value: serde_json::json!("new"),
            },
            PatchOp::SetTaskStatus {
                target: task,
                status: TaskStatus::Done,
            },
            PatchOp::ReplaceTableRow {
                target: row,
                markdown: "| A | closed |".into(),
            },
        ],
    };
    let outcome = patch.apply(&document).unwrap();
    let replay = patch.apply(&document).unwrap();
    assert_eq!(replay.document.source(), outcome.document.source());
    assert_eq!(replay.document.revision(), outcome.document.revision());
    assert_eq!(replay.receipts, outcome.receipts);
    assert_eq!(
        outcome.document.source(),
        "---\nstatus: new\n---\n\n# Work\n\n- [x] task\n\n| Name | State |\n| --- | --- |\n| A | closed |\n"
    );
    assert_eq!(outcome.receipts.len(), 3);
    assert!(matches!(
        outcome.receipts.as_slice(),
        [
            mdtools::patch::PatchReceipt::SetFrontmatter { .. },
            mdtools::patch::PatchReceipt::SetTaskStatus { .. },
            mdtools::patch::PatchReceipt::ReplaceTableRow { .. }
        ]
    ));
    for receipt in &outcome.receipts {
        let wire = serde_json::to_value(receipt).unwrap();
        assert_eq!(
            serde_json::from_value::<mdtools::patch::PatchReceipt>(wire).unwrap(),
            *receipt
        );
    }
}

#[test]
fn grouped_frontmatter_receipts_preserve_interleaved_patch_order() {
    let document =
        Document::parse_for_frontmatter_mutation("---\na: old\nb: old\n---\n\n- [ ] task\n")
            .unwrap();
    let field = |name: &str| {
        let resolved = document
            .resolve(&TargetAddress::FrontmatterField {
                path: vec![name.into()],
            })
            .unwrap();
        FrontmatterPatchTarget::try_from(resolved.snapshot()).unwrap()
    };
    let task = TaskPatchTarget::try_from(
        document
            .map()
            .unwrap()
            .iter()
            .find(|snapshot| snapshot.kind == TargetKind::Task)
            .unwrap(),
    )
    .unwrap();
    let outcome = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::SetFrontmatter {
                target: field("a"),
                value: serde_json::json!("new_a"),
            },
            PatchOp::SetTaskStatus {
                target: task,
                status: TaskStatus::Done,
            },
            PatchOp::SetFrontmatter {
                target: field("b"),
                value: serde_json::json!("new_b"),
            },
        ],
    }
    .apply(&document)
    .unwrap();
    assert!(matches!(
        outcome.receipts.as_slice(),
        [
            mdtools::patch::PatchReceipt::SetFrontmatter { .. },
            mdtools::patch::PatchReceipt::SetTaskStatus { .. },
            mdtools::patch::PatchReceipt::SetFrontmatter { .. }
        ]
    ));
}

#[test]
fn frontmatter_container_conflict_and_stale_guard_fail_before_output() {
    let document =
        Document::parse_for_frontmatter_mutation("---\na: scalar\n---\n\nbody\n").unwrap();
    let nested = document
        .resolve(&TargetAddress::FrontmatterField {
            path: vec!["a".into(), "b".into()],
        })
        .unwrap();
    let conflict = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::SetFrontmatter {
            target: FrontmatterPatchTarget::try_from(nested.snapshot()).unwrap(),
            value: serde_json::json!("new"),
        }],
    };
    assert!(matches!(
        conflict.apply(&document),
        Err(mdtools::core_error::CoreError::FrontmatterFieldConflict { .. })
    ));

    let field = document
        .resolve(&TargetAddress::FrontmatterField {
            path: vec!["a".into()],
        })
        .unwrap();
    let mut stale = FrontmatterPatchTarget::try_from(field.snapshot()).unwrap();
    stale.guard.etag = mdtools::fingerprint::TargetEtag::for_bytes(b"stale");
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::SetFrontmatter {
            target: stale,
            value: serde_json::json!("new"),
        }],
    };
    assert!(matches!(
        patch.apply(&document),
        Err(mdtools::core_error::CoreError::TargetAuthorityMismatch { .. })
    ));
    assert_eq!(document.source(), "---\na: scalar\n---\n\nbody\n");
}

#[test]
fn overlapping_patch_is_rejected() {
    let document = Document::parse("# Work\n\n- [ ] task\n").unwrap();
    let map = document.map().unwrap();
    let block = map
        .iter()
        .find(|snapshot| {
            matches!(
                snapshot.summary,
                TargetSummary::Block {
                    kind: BlockKind::List,
                    ..
                }
            )
        })
        .unwrap();
    let task = map
        .iter()
        .find(|snapshot| snapshot.kind == TargetKind::Task)
        .unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::ReplaceBlock {
                target: ReplaceBlockTarget::try_from(block).unwrap(),
                markdown: "replacement".into(),
            },
            PatchOp::SetTaskStatus {
                target: TaskPatchTarget::try_from(task).unwrap(),
                status: TaskStatus::Done,
            },
        ],
    };
    assert!(
        matches!(patch.apply(&document), Err(mdtools::core_error::CoreError::PatchInvariant(message)) if message.contains("overlap"))
    );
    assert_eq!(document.source(), "# Work\n\n- [ ] task\n");
}

#[test]
fn semantic_block_and_task_overlap_rejects_even_when_byte_edits_do_not() {
    let document = Document::parse("- [ ] old\n").unwrap();
    let map = document.map().unwrap();
    let block = map
        .iter()
        .find(|snapshot| snapshot.kind == TargetKind::Block)
        .unwrap();
    let task = map
        .iter()
        .find(|snapshot| snapshot.kind == TargetKind::Task)
        .unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::ReplaceBlock {
                target: ReplaceBlockTarget::try_from(block).unwrap(),
                markdown: "- [ ] new".into(),
            },
            PatchOp::SetTaskStatus {
                target: TaskPatchTarget::try_from(task).unwrap(),
                status: TaskStatus::Done,
            },
        ],
    };

    assert!(matches!(
        patch.apply(&document),
        Err(mdtools::core_error::CoreError::PatchInvariant(message))
            if message.contains("overlap")
    ));
    assert_eq!(document.source(), "- [ ] old\n");
}

#[test]
fn move_interval_conflicts_with_an_intermediate_nochange_target() {
    let document = Document::parse("aaa\n\nbbb\n\nccc\n").unwrap();
    let blocks = document
        .map()
        .unwrap()
        .into_iter()
        .filter(|snapshot| snapshot.kind == TargetKind::Block)
        .collect::<Vec<_>>();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::MoveBlock {
                source: ReplaceBlockTarget::try_from(&blocks[0]).unwrap(),
                destination: ReplaceBlockTarget::try_from(&blocks[2]).unwrap(),
                position: mdtools::patch::RelativePosition::After,
            },
            PatchOp::ReplaceBlock {
                target: ReplaceBlockTarget::try_from(&blocks[1]).unwrap(),
                markdown: "bbb".into(),
            },
        ],
    };

    assert!(matches!(
        patch.apply(&document),
        Err(mdtools::core_error::CoreError::PatchInvariant(message))
            if message.contains("overlap")
    ));
}

#[test]
fn nested_task_checkbox_updates_are_semantically_disjoint() {
    let document = Document::parse("- [ ] parent\n  - [ ] child\n").unwrap();
    let tasks = document
        .map()
        .unwrap()
        .into_iter()
        .filter(|snapshot| snapshot.kind == TargetKind::Task)
        .collect::<Vec<_>>();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: tasks
            .iter()
            .map(|task| PatchOp::SetTaskStatus {
                target: TaskPatchTarget::try_from(task).unwrap(),
                status: TaskStatus::Done,
            })
            .collect(),
    };

    let outcome = patch.apply(&document).unwrap();
    assert_eq!(outcome.document.source(), "- [x] parent\n  - [x] child\n");
}

#[test]
fn sibling_frontmatter_paths_are_semantically_disjoint() {
    let document =
        Document::parse_for_frontmatter_mutation("---\na: old\nb: old\n---\n\nbody\n").unwrap();
    let target = |path: &str| {
        let resolved = document
            .resolve(&TargetAddress::FrontmatterField {
                path: vec![path.into()],
            })
            .unwrap();
        FrontmatterPatchTarget::try_from(resolved.snapshot()).unwrap()
    };
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::SetFrontmatter {
                target: target("a"),
                value: serde_json::json!("new_a"),
            },
            PatchOp::SetFrontmatter {
                target: target("b"),
                value: serde_json::json!("new_b"),
            },
        ],
    };

    let outcome = patch.apply(&document).unwrap();
    assert!(outcome.document.source().contains("a: new_a"));
    assert!(outcome.document.source().contains("b: new_b"));
}

#[test]
fn equal_or_prefix_related_frontmatter_paths_conflict() {
    let document =
        Document::parse_for_frontmatter_mutation("---\na:\n  b: old\n---\n\nbody\n").unwrap();
    let target = |path: Vec<&str>| {
        let resolved = document
            .resolve(&TargetAddress::FrontmatterField {
                path: path.into_iter().map(str::to_string).collect(),
            })
            .unwrap();
        FrontmatterPatchTarget::try_from(resolved.snapshot()).unwrap()
    };
    for operations in [
        vec![
            PatchOp::SetFrontmatter {
                target: target(vec!["a"]),
                value: serde_json::json!({"b": "new"}),
            },
            PatchOp::SetFrontmatter {
                target: target(vec!["a", "b"]),
                value: serde_json::json!("new"),
            },
        ],
        vec![
            PatchOp::SetFrontmatter {
                target: target(vec!["a", "b"]),
                value: serde_json::json!("new"),
            },
            PatchOp::DeleteFrontmatter {
                target: target(vec!["a", "b"]),
            },
        ],
    ] {
        let patch = Patch {
            base_revision: document.revision().clone(),
            operations,
        };
        assert!(matches!(
            patch.apply(&document),
            Err(mdtools::core_error::CoreError::PatchInvariant(message))
                if message.contains("overlap")
        ));
    }
}

#[test]
fn moving_a_mid_document_footnote_definition_uses_source_order() {
    let source = "# H\n\nref[^1]\n\n[^1]: note\n\ntail\n";
    let document = Document::parse(source).unwrap();
    let map = document.map().unwrap();
    let footnote = map
        .iter()
        .find(|snapshot| {
            snapshot.kind == TargetKind::Block
                && snapshot
                    .selection_span
                    .is_some_and(|span| document.slice(&span).unwrap().contains("[^1]:"))
        })
        .unwrap();
    let tail = map
        .iter()
        .find(|snapshot| matches!(&snapshot.summary, TargetSummary::Block { preview, .. } if preview == "tail"))
        .unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::MoveBlock {
            source: ReplaceBlockTarget::try_from(footnote).unwrap(),
            destination: ReplaceBlockTarget::try_from(tail).unwrap(),
            position: mdtools::patch::RelativePosition::After,
        }],
    };

    let result = std::panic::catch_unwind(|| patch.apply(&document));
    assert!(result.is_ok(), "footnote move must not panic");
    let outcome = result.unwrap().unwrap();
    assert!(
        outcome.document.source().find("tail").unwrap()
            < outcome.document.source().find("[^1]: note").unwrap()
    );
}

#[test]
fn block_move_rejects_candidate_that_absorbs_an_untouched_html_bystander() {
    let document = Document::parse("a\n<div>x</div>\n\nb\n").unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::MoveBlock {
            source: ReplaceBlockTarget::try_from(&block_with(&document, "a")).unwrap(),
            destination: ReplaceBlockTarget::try_from(&block_with(&document, "b")).unwrap(),
            position: mdtools::patch::RelativePosition::After,
        }],
    };
    assert!(matches!(
        patch.apply(&document),
        Err(mdtools::core_error::CoreError::PatchInvariant(message))
            if message.contains("closure")
    ));
}

#[test]
fn section_move_composes_with_an_unrelated_task_and_preserves_bystanders() {
    let source = "- [ ] task\n\n# Doc\n\n## A\n\na\n\n## B\n\n<div>x</div>\n\nb\n\n## C\n\nc\n";
    let document = Document::parse(source).unwrap();
    let map = document.map().unwrap();
    let section = |heading: &str| {
        HeadingPatchTarget::try_from(
            map.iter()
                .find(|snapshot| matches!(&snapshot.summary, TargetSummary::Section { heading: value, .. } if value == heading))
                .unwrap(),
        )
        .unwrap()
    };
    let task = TaskPatchTarget::try_from(
        map.iter()
            .find(|snapshot| snapshot.kind == TargetKind::Task)
            .unwrap(),
    )
    .unwrap();
    let outcome = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::SetTaskStatus {
                target: task,
                status: TaskStatus::Done,
            },
            PatchOp::MoveSection {
                source: section("A"),
                destination: section("C"),
                position: SectionMovePosition::AfterSibling,
                keep_level: true,
            },
        ],
    }
    .apply(&document)
    .unwrap();
    assert!(outcome.document.source().contains("- [x] task"));
    assert!(outcome.document.source().contains("<div>x</div>\n\nb"));
    assert!(
        outcome.document.source().find("## C").unwrap()
            < outcome.document.source().find("## A").unwrap()
    );
}

#[test]
fn nochange_operations_still_claim_their_semantic_target() {
    let document = Document::parse("- [ ] old\n").unwrap();
    let map = document.map().unwrap();
    let block = map
        .iter()
        .find(|snapshot| snapshot.kind == TargetKind::Block)
        .unwrap();
    let task = map
        .iter()
        .find(|snapshot| snapshot.kind == TargetKind::Task)
        .unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::ReplaceBlock {
                target: ReplaceBlockTarget::try_from(block).unwrap(),
                markdown: "- [ ] old".into(),
            },
            PatchOp::SetTaskStatus {
                target: TaskPatchTarget::try_from(task).unwrap(),
                status: TaskStatus::Done,
            },
        ],
    };

    assert!(matches!(
        patch.apply(&document),
        Err(mdtools::core_error::CoreError::PatchInvariant(message))
            if message.contains("overlap")
    ));
}

#[test]
fn task_receipt_identity_resolves_after_an_earlier_insertion() {
    let document = Document::parse("one\n\n- [ ] task\n").unwrap();
    let map = document.map().unwrap();
    let first = ReplaceBlockTarget::try_from(
        map.iter()
            .find(|snapshot| matches!(&snapshot.summary, TargetSummary::Block { preview, .. } if preview == "one"))
            .unwrap(),
    )
    .unwrap();
    let task = TaskPatchTarget::try_from(
        map.iter()
            .find(|snapshot| snapshot.kind == TargetKind::Task)
            .unwrap(),
    )
    .unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::InsertBlock {
                target: BlockInsertionTarget::Before { anchor: first },
                markdown: "zero".into(),
            },
            PatchOp::SetTaskStatus {
                target: task,
                status: TaskStatus::Done,
            },
        ],
    };

    let outcome = patch.apply(&document).unwrap();
    let mdtools::patch::PatchReceipt::SetTaskStatus {
        outcome: task_outcome,
    } = &outcome.receipts[1]
    else {
        panic!("second receipt is set_task_status")
    };
    let (before, after) = match task_outcome {
        mdtools::patch::SetTaskOutcome::NoChange { before, after }
        | mdtools::patch::SetTaskOutcome::Replaced { before, after } => (before, after),
    };
    assert_eq!(before.revision, *document.revision());
    assert_eq!(after.revision, *outcome.document.revision());
    assert!(outcome
        .document
        .resolve(&TargetAddress::Task {
            block: after.block.clone(),
            path: after.path.clone(),
        })
        .is_ok());
}

#[test]
fn block_move_planning_preserves_strict_frontmatter_policy() {
    let source = "---\ntitle: [\n---\n# H\n\none\n\ntwo\n";
    let document = Document::parse_for_frontmatter(source).unwrap();
    let section = mdtools::target::SectionAddress::Heading {
        path: vec![mdtools::target::HeadingAddressSegment {
            text: "H".into(),
            occurrence: 1,
        }],
    };
    let first = document
        .resolve(&TargetAddress::Block {
            block: mdtools::target::BlockAddress {
                section: section.clone(),
                ordinal: 0,
            },
        })
        .unwrap();
    let second = document
        .resolve(&TargetAddress::Block {
            block: mdtools::target::BlockAddress {
                section,
                ordinal: 1,
            },
        })
        .unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::MoveBlock {
            source: ReplaceBlockTarget::try_from(first.snapshot()).unwrap(),
            destination: ReplaceBlockTarget::try_from(second.snapshot()).unwrap(),
            position: mdtools::patch::RelativePosition::After,
        }],
    };

    let outcome = patch.apply(&document).unwrap();
    assert_eq!(
        outcome.document.source(),
        "---\ntitle: [\n---\n# H\n\ntwo\n\none\n"
    );
    assert!(outcome.document.has_frontmatter());
}

#[test]
fn operations_cannot_depend_on_state_created_by_earlier_operations() {
    let document = Document::parse("body\n").unwrap();
    let block = document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| snapshot.kind == TargetKind::Block)
        .unwrap();
    let mut nonexistent = TaskPatchTarget {
        block: match &block.address {
            TargetAddress::Block { block } => block.clone(),
            _ => unreachable!(),
        },
        path: vec![99],
        revision: document.revision().clone(),
        guard: mdtools::patch::SelectionGuard {
            span: block.selection_span.unwrap(),
            etag: match &block.guard {
                mdtools::target::GuardAuthority::Selection { etag, .. } => etag.clone(),
                _ => unreachable!(),
            },
        },
    };
    nonexistent.path = vec![0];
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::ReplaceBlock {
                target: ReplaceBlockTarget::try_from(&block).unwrap(),
                markdown: "- [ ] new task".into(),
            },
            PatchOp::SetTaskStatus {
                target: nonexistent,
                status: TaskStatus::Done,
            },
        ],
    };
    assert!(matches!(
        patch.apply(&document),
        Err(mdtools::core_error::CoreError::TargetNotFound { .. })
    ));
}

#[test]
fn insertion_before_replacement_keeps_the_replaced_target_receipt_bound() {
    let document = Document::parse("one\n\ntwo\n").unwrap();
    let blocks = document
        .map()
        .unwrap()
        .into_iter()
        .filter(|snapshot| snapshot.kind == TargetKind::Block)
        .collect::<Vec<_>>();
    let first = ReplaceBlockTarget::try_from(&blocks[0]).unwrap();
    let second = ReplaceBlockTarget::try_from(&blocks[1]).unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::InsertBlock {
                target: BlockInsertionTarget::Before {
                    anchor: first.clone(),
                },
                markdown: "zero".into(),
            },
            PatchOp::ReplaceBlock {
                target: second,
                markdown: "changed".into(),
            },
        ],
    };

    let outcome = patch.apply(&document).unwrap();
    assert_eq!(outcome.document.source(), "zero\n\none\n\nchanged\n");
    assert_eq!(outcome.receipts.len(), 2);
    assert_eq!(
        outcome.receipts[1].replace_block_before().unwrap().preview,
        "two"
    );
    assert_eq!(
        outcome.receipts[1].replace_block_after().unwrap().preview,
        "changed"
    );
    let before = outcome.receipts[1].replace_block_before().unwrap();
    let after = outcome.receipts[1].replace_block_after().unwrap();
    assert_eq!(before.address.ordinal, 1);
    assert_eq!(before.revision, *document.revision());
    assert_eq!(after.address.ordinal, 2);
    assert_eq!(after.revision, *outcome.document.revision());
    assert!(outcome
        .document
        .resolve(&TargetAddress::Block {
            block: after.address.clone(),
        })
        .is_ok());
}

#[test]
fn shifted_nochange_block_receipt_has_distinct_before_and_after_identities() {
    let document = Document::parse("one\n\ntwo\n").unwrap();
    let blocks = document
        .map()
        .unwrap()
        .into_iter()
        .filter(|snapshot| snapshot.kind == TargetKind::Block)
        .collect::<Vec<_>>();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::InsertBlock {
                target: BlockInsertionTarget::Before {
                    anchor: ReplaceBlockTarget::try_from(&blocks[0]).unwrap(),
                },
                markdown: "zero".into(),
            },
            PatchOp::ReplaceBlock {
                target: ReplaceBlockTarget::try_from(&blocks[1]).unwrap(),
                markdown: "two".into(),
            },
        ],
    };

    let outcome = patch.apply(&document).unwrap();
    let before = outcome.receipts[1].replace_block_before().unwrap();
    let after = outcome.receipts[1].replace_block_after().unwrap();
    assert_eq!(
        outcome.receipts[1].disposition(),
        mdtools::MutationDisposition::NoChange
    );
    assert_eq!(before.address.ordinal, 1);
    assert_eq!(before.revision, *document.revision());
    assert_eq!(after.address.ordinal, 2);
    assert_eq!(after.revision, *outcome.document.revision());
    assert_ne!(before.guard.span, after.guard.span);
}

#[test]
fn table_row_receipt_rebinds_after_an_earlier_block_insertion() {
    let document = Document::parse("lead\n\n| Name |\n| --- |\n| old |\n").unwrap();
    let map = document.map().unwrap();
    let lead = ReplaceBlockTarget::try_from(
        map.iter()
            .find(|snapshot| matches!(&snapshot.summary, TargetSummary::Block { preview, .. } if preview == "lead"))
            .unwrap(),
    )
    .unwrap();
    let row = TableRowPatchTarget::try_from(
        map.iter()
            .find(|snapshot| snapshot.kind == TargetKind::TableRow)
            .unwrap(),
    )
    .unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::InsertBlock {
                target: BlockInsertionTarget::Before { anchor: lead },
                markdown: "zero".into(),
            },
            PatchOp::ReplaceTableRow {
                target: row,
                markdown: "| new |".into(),
            },
        ],
    };

    let outcome = patch.apply(&document).unwrap();
    let mdtools::patch::PatchReceipt::ReplaceTableRow {
        outcome: row_outcome,
    } = &outcome.receipts[1]
    else {
        panic!("second receipt is replace_table_row")
    };
    let (before, after) = match row_outcome {
        mdtools::patch::ReplaceTableRowOutcome::NoChange { before, after }
        | mdtools::patch::ReplaceTableRowOutcome::Replaced { before, after } => (before, after),
    };
    assert_eq!(before.table.ordinal, 1);
    assert_eq!(after.table.ordinal, 2);
    assert_eq!(after.revision, *outcome.document.revision());
    assert!(outcome
        .document
        .resolve(&TargetAddress::TableRow {
            table: after.table.clone(),
            row: after.row,
        })
        .is_ok());
}

#[test]
fn identical_blocks_use_addresses_not_content_identity() {
    let document = Document::parse("same\n\nsame\n").unwrap();
    let blocks = document
        .map()
        .unwrap()
        .into_iter()
        .filter(|snapshot| snapshot.kind == TargetKind::Block)
        .collect::<Vec<_>>();
    assert_eq!(blocks.len(), 2);
    let mdtools::target::GuardAuthority::Selection {
        etag: first_etag, ..
    } = &blocks[0].guard
    else {
        unreachable!()
    };
    let mdtools::target::GuardAuthority::Selection {
        etag: second_etag, ..
    } = &blocks[1].guard
    else {
        unreachable!()
    };
    assert_eq!(first_etag, second_etag);

    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::ReplaceBlock {
            target: ReplaceBlockTarget::try_from(&blocks[1]).unwrap(),
            markdown: "changed".into(),
        }],
    };
    let outcome = patch.apply(&document).unwrap();
    assert_eq!(outcome.document.source(), "same\n\nchanged\n");
}

#[test]
fn every_guard_is_checked_before_any_payload_is_processed() {
    let document = Document::parse("body\n").unwrap();
    let block = document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| snapshot.kind == TargetKind::Block)
        .unwrap();
    let valid = ReplaceBlockTarget::try_from(&block).unwrap();
    let mut stale = valid.clone();
    stale.guard.etag = mdtools::fingerprint::TargetEtag::for_bytes(b"stale");
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::ReplaceBlock {
                target: valid,
                markdown: "# heading\n\nsecond block".into(),
            },
            PatchOp::ReplaceBlock {
                target: stale,
                markdown: "unused".into(),
            },
        ],
    };

    assert!(matches!(
        patch.apply(&document),
        Err(mdtools::core_error::CoreError::TargetAuthorityMismatch { .. })
    ));
    assert_eq!(document.source(), "body\n");
}

#[test]
fn overlapping_move_and_replacement_reject_the_whole_patch() {
    let document = Document::parse("one\n\ntwo\n\nthree\n\nfour\n").unwrap();
    let blocks = document
        .map()
        .unwrap()
        .into_iter()
        .filter(|snapshot| snapshot.kind == TargetKind::Block)
        .collect::<Vec<_>>();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::MoveBlock {
                source: ReplaceBlockTarget::try_from(&blocks[0]).unwrap(),
                destination: ReplaceBlockTarget::try_from(&blocks[2]).unwrap(),
                position: mdtools::patch::RelativePosition::After,
            },
            PatchOp::ReplaceBlock {
                target: ReplaceBlockTarget::try_from(&blocks[1]).unwrap(),
                markdown: "changed".into(),
            },
        ],
    };

    assert!(matches!(
        patch.apply(&document),
        Err(mdtools::core_error::CoreError::PatchInvariant(message))
            if message.contains("overlap")
    ));
    assert_eq!(document.source(), "one\n\ntwo\n\nthree\n\nfour\n");
}

#[test]
fn all_nochange_patch_skips_write() {
    let source = "---\nstatus: old\n---\n\n- [ ] task\n";
    let document = Document::parse_for_frontmatter_mutation(source).unwrap();
    let map = document.map().unwrap();
    let task = TaskPatchTarget::try_from(
        map.iter()
            .find(|snapshot| snapshot.kind == TargetKind::Task)
            .unwrap(),
    )
    .unwrap();
    let field = document
        .resolve(&TargetAddress::FrontmatterField {
            path: vec!["status".into()],
        })
        .unwrap();
    let field = FrontmatterPatchTarget::try_from(field.snapshot()).unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::SetTaskStatus {
                target: task,
                status: TaskStatus::Pending,
            },
            PatchOp::SetFrontmatter {
                target: field,
                value: serde_json::json!("old"),
            },
        ],
    };

    let outcome = patch.apply(&document).unwrap();
    assert_eq!(outcome.document.source(), source);
    assert_eq!(outcome.document.revision(), document.revision());
    assert!(outcome
        .receipts
        .iter()
        .all(|receipt| receipt.disposition() == mdtools::MutationDisposition::NoChange));
}

#[test]
fn disjoint_crlf_operations_preserve_document_line_endings() {
    let source = "- [ ] task\r\n\r\n| Name |\r\n| --- |\r\n| old |\r\n";
    let document = Document::parse(source).unwrap();
    let map = document.map().unwrap();
    let task = TaskPatchTarget::try_from(
        map.iter()
            .find(|snapshot| snapshot.kind == TargetKind::Task)
            .unwrap(),
    )
    .unwrap();
    let row = TableRowPatchTarget::try_from(
        map.iter()
            .find(|snapshot| snapshot.kind == TargetKind::TableRow)
            .unwrap(),
    )
    .unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::SetTaskStatus {
                target: task,
                status: TaskStatus::Done,
            },
            PatchOp::ReplaceTableRow {
                target: row,
                markdown: "| new |".into(),
            },
        ],
    };

    let outcome = patch.apply(&document).unwrap();
    assert_eq!(
        outcome.document.source(),
        "- [x] task\r\n\r\n| Name |\r\n| --- |\r\n| new |\r\n"
    );
}

#[test]
fn disjoint_multibyte_replacements_compile_on_utf8_boundaries() {
    let document = Document::parse("éclair\n\n世界\n").unwrap();
    let blocks = document
        .map()
        .unwrap()
        .into_iter()
        .filter(|snapshot| snapshot.kind == TargetKind::Block)
        .collect::<Vec<_>>();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::ReplaceBlock {
                target: ReplaceBlockTarget::try_from(&blocks[0]).unwrap(),
                markdown: "êclair".into(),
            },
            PatchOp::ReplaceBlock {
                target: ReplaceBlockTarget::try_from(&blocks[1]).unwrap(),
                markdown: "世間".into(),
            },
        ],
    };

    let outcome = patch.apply(&document).unwrap();
    assert_eq!(outcome.document.source(), "êclair\n\n世間\n");
    assert_eq!(
        outcome.receipts[0].replace_block_after().unwrap().preview,
        "êclair"
    );
    assert_eq!(
        outcome.receipts[1].replace_block_after().unwrap().preview,
        "世間"
    );
}
