use std::collections::HashSet;

use mdtools::document::Document;
use mdtools::patch::{
    BlockInsertionTarget, DocumentEdge, FrontmatterPatchTarget, InsertBlockOutcome, Patch, PatchOp,
    PatchReceipt, ReplaceBlockOutcome, ReplaceBlockTarget, ReplaceTableRowOutcome,
    SetFrontmatterOutcome, SetTaskOutcome, TableRowPatchTarget, TaskPatchTarget,
};
use mdtools::target::{TargetAddress, TargetKind, TargetSnapshot, TargetSummary};
use mdtools::{BlockKind, MutationDisposition, TaskStatus};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

const PROPERTY_CASES: u32 = 128;

#[derive(Clone, Debug)]
struct DocumentCase {
    crlf: bool,
    final_newline: bool,
    frontmatter: bool,
    setext_root: bool,
    extra_blank_line: bool,
    trailing_spaces: bool,
    referenced_footnote: bool,
    atom: String,
}

#[derive(Clone, Debug)]
struct TransactionCase {
    operation_mask: u8,
    insertion_before: bool,
    second_task: bool,
    second_row: bool,
    atom: String,
}

impl DocumentCase {
    fn render(&self) -> String {
        let root = if self.setext_root {
            "Root\n===="
        } else {
            "# Root"
        };
        let trailing = if self.trailing_spaces { "  " } else { "" };
        let extra_blank = if self.extra_blank_line { "\n" } else { "" };
        let mut source = String::new();
        if self.frontmatter {
            source.push_str("---\nk0: old\nk1: stable\n---\n\n");
        }
        let footnote_reference = if self.referenced_footnote {
            format!("reference-{}[^note]\n\n", self.atom)
        } else {
            String::new()
        };
        source.push_str(&format!(
            "preamble-{atom}{trailing}\n\n{extra_blank}{root}\n\nparagraph-main-{atom}{trailing}\n\n## Duplicate\n\nparagraph-stable-{atom}{trailing}\n\n## Duplicate\n\n- [ ] parent-{atom}{trailing}\n  - [ ] child-{atom}\n\n{footnote_reference}[^note]: footnote-{atom}\n\n| Name | State |\n| --- | --- |\n| row-{atom} | open |\n| stable-{atom} | fixed |\n",
            atom = self.atom,
        ));
        if !self.final_newline {
            source.pop();
        }
        if self.crlf {
            source = source.replace('\n', "\r\n");
        }
        source
    }

    fn parse(&self, source: String) -> Document {
        let rendered = source.clone();
        if self.frontmatter {
            Document::parse_for_frontmatter_mutation(source)
        } else {
            Document::parse(source)
        }
        .unwrap_or_else(|error| {
            panic!("generated document must parse: {self:?}: {error}\nsource={rendered:?}")
        })
    }

    fn document(&self) -> Document {
        self.parse(self.render())
    }

    fn untouched_boundary_sentinels(&self) -> [String; 2] {
        let line_ending = if self.crlf { "\r\n" } else { "\n" };
        let trailing = if self.trailing_spaces { "  " } else { "" };
        let extra_blank = if self.extra_blank_line {
            line_ending
        } else {
            ""
        };
        let root = if self.setext_root {
            format!("Root{line_ending}====")
        } else {
            "# Root".into()
        };
        [
            format!(
                "preamble-{}{trailing}{line_ending}{line_ending}{extra_blank}{root}",
                self.atom
            ),
            format!(
                "paragraph-stable-{}{trailing}{line_ending}{line_ending}",
                self.atom
            ),
        ]
    }
}

fn document_case() -> impl Strategy<Value = DocumentCase> {
    let atom = prop_oneof![
        Just("alpha".to_string()),
        Just("bravo".to_string()),
        Just("éclair".to_string()),
        Just("世界".to_string()),
    ];
    (
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        atom,
    )
        .prop_map(
            |(
                crlf,
                final_newline,
                frontmatter,
                setext_root,
                extra_blank_line,
                trailing_spaces,
                referenced_footnote,
                atom,
            )| DocumentCase {
                crlf,
                final_newline,
                frontmatter,
                setext_root,
                extra_blank_line,
                trailing_spaces,
                referenced_footnote,
                atom,
            },
        )
}

fn transaction_case() -> impl Strategy<Value = TransactionCase> {
    let atom = prop_oneof![
        Just("november".to_string()),
        Just("résumé".to_string()),
        Just("更新".to_string()),
    ];
    (1u8..64, any::<bool>(), any::<bool>(), any::<bool>(), atom).prop_map(
        |(operation_mask, insertion_before, second_task, second_row, atom)| TransactionCase {
            operation_mask,
            insertion_before,
            second_task,
            second_row,
            atom,
        },
    )
}

fn find_snapshot(
    document: &Document,
    predicate: impl Fn(&TargetSnapshot) -> bool,
) -> TargetSnapshot {
    document
        .map()
        .expect("generated document maps")
        .into_iter()
        .find(predicate)
        .expect("generated document contains requested target")
}

fn snapshots(document: &Document, kind: TargetKind) -> Vec<TargetSnapshot> {
    document
        .map()
        .expect("generated document maps")
        .into_iter()
        .filter(|snapshot| snapshot.kind == kind)
        .collect()
}

impl TransactionCase {
    fn patch(&self, document: &Document) -> Patch {
        let mut operations = Vec::new();
        if self.operation_mask & 1 != 0 {
            let block = find_snapshot(
                document,
                |snapshot| matches!(&snapshot.summary, TargetSummary::Block { preview, .. } if preview.starts_with("paragraph-main-")),
            );
            operations.push(PatchOp::ReplaceBlock {
                target: ReplaceBlockTarget::try_from(&block).expect("block evidence refines"),
                markdown: format!("paragraph-new-{}", self.atom),
            });
        }
        if self.operation_mask & 2 != 0 {
            let tasks = snapshots(document, TargetKind::Task);
            let task = &tasks[usize::from(self.second_task)];
            operations.push(PatchOp::SetTaskStatus {
                target: TaskPatchTarget::try_from(task).expect("task evidence refines"),
                status: TaskStatus::Done,
            });
        }
        if self.operation_mask & 4 != 0 {
            let field = document
                .resolve(&TargetAddress::FrontmatterField {
                    path: vec!["k0".into()],
                })
                .expect("frontmatter field is exactly addressable");
            operations.push(PatchOp::SetFrontmatter {
                target: FrontmatterPatchTarget::try_from(field.snapshot())
                    .expect("frontmatter evidence refines"),
                value: serde_json::json!(format!("frontmatter-new-{}", self.atom)),
            });
        }
        if self.operation_mask & 8 != 0 {
            let rows = snapshots(document, TargetKind::TableRow);
            let row = &rows[usize::from(self.second_row)];
            operations.push(PatchOp::ReplaceTableRow {
                target: TableRowPatchTarget::try_from(row).expect("row evidence refines"),
                markdown: if self.second_row {
                    format!("| stable-new-{} | changed |", self.atom)
                } else {
                    format!("| row-new-{} | closed |", self.atom)
                },
            });
        }
        if self.operation_mask & 16 != 0 {
            let final_row_replacement = self.operation_mask & 8 != 0 && self.second_row;
            let target = if self.insertion_before || final_row_replacement {
                let list = find_snapshot(document, |snapshot| {
                    matches!(
                        snapshot.summary,
                        TargetSummary::Block {
                            kind: BlockKind::List,
                            ..
                        }
                    )
                });
                BlockInsertionTarget::Before {
                    anchor: ReplaceBlockTarget::try_from(&list)
                        .expect("insertion anchor evidence refines"),
                }
            } else {
                BlockInsertionTarget::DocumentEdge {
                    edge: DocumentEdge::End,
                    revision: document.revision().clone(),
                }
            };
            operations.push(PatchOp::InsertBlock {
                target,
                markdown: format!("tail-new-{}", self.atom),
            });
        }
        if self.operation_mask & 32 != 0 {
            let field = document
                .resolve(&TargetAddress::FrontmatterField {
                    path: vec!["k1".into()],
                })
                .expect("sibling frontmatter field is exactly addressable");
            operations.push(PatchOp::SetFrontmatter {
                target: FrontmatterPatchTarget::try_from(field.snapshot())
                    .expect("sibling frontmatter evidence refines"),
                value: serde_json::json!(format!("frontmatter-sibling-{}", self.atom)),
            });
        }
        Patch {
            base_revision: document.revision().clone(),
            operations,
        }
    }
}

fn assert_document_contract(document: &Document) {
    let mapped = document.map().expect("generated document maps");
    let mut addresses = HashSet::<TargetAddress>::new();
    let mut last_selection_start = 0;
    for snapshot in mapped {
        assert!(
            addresses.insert(snapshot.address.clone()),
            "canonical address occurs more than once: {}",
            snapshot.address
        );
        if let Some(span) = snapshot.selection_span {
            assert!(
                span.byte_start >= last_selection_start,
                "mapped selections are not source ordered: {} < {} for {}",
                span.byte_start,
                last_selection_start,
                snapshot.address
            );
            last_selection_start = span.byte_start;
            document.slice(&span).expect("selection span slices source");
        }
        let resolved = document
            .resolve(&snapshot.address)
            .expect("mapped address resolves");
        assert_eq!(resolved.snapshot(), &snapshot);
        assert_eq!(
            document
                .read_target(&resolved)
                .expect("resolved target has a typed read")
                .snapshot(),
            &snapshot
        );
    }
}

fn assert_generated_inventory(document: &Document, case: &DocumentCase) {
    let mapped = document.map().expect("generated inventory maps");
    let expected_footnotes = usize::from(case.referenced_footnote);
    assert_eq!(
        mapped
            .iter()
            .filter(|snapshot| matches!(&snapshot.summary, TargetSummary::Section { heading, .. } if heading == "Duplicate"))
            .count(),
        2,
        "both duplicate headings must remain independently addressable: {case:?}"
    );
    assert_eq!(
        mapped
            .iter()
            .filter(|snapshot| snapshot.kind == TargetKind::Task)
            .count(),
        2,
        "parent and child tasks must remain mapped: {case:?}"
    );
    assert_eq!(
        mapped
            .iter()
            .filter(|snapshot| snapshot.kind == TargetKind::TableRow)
            .count(),
        2,
        "both table rows must remain mapped: {case:?}"
    );
    assert_eq!(
        mapped
            .iter()
            .filter(|snapshot| {
                matches!(
                    snapshot.summary,
                    TargetSummary::Block {
                        kind: BlockKind::FootnoteDefinition,
                        ..
                    }
                )
            })
            .count(),
        expected_footnotes,
        "footnote target inventory must follow reference state: {case:?}"
    );
    assert!(
        document
            .source()
            .contains(&format!("[^note]: footnote-{}", case.atom)),
        "parser-unrepresented footnote bytes must remain in source: {case:?}"
    );
}

fn assert_block_resolves(document: &Document, address: &mdtools::target::BlockAddress) {
    document
        .resolve(&TargetAddress::Block {
            block: address.clone(),
        })
        .expect("receipt block identity resolves");
}

fn assert_receipt_bindings(base: &Document, result: &Document, receipt: &PatchReceipt) {
    match receipt {
        PatchReceipt::ReplaceBlock { outcome } => {
            let (before, after) = match outcome {
                ReplaceBlockOutcome::NoChange { before, after }
                | ReplaceBlockOutcome::Replaced { before, after } => (before, after),
                ReplaceBlockOutcome::Deleted { .. } => panic!("generated replacement survives"),
            };
            assert_eq!(&before.revision, base.revision());
            assert_eq!(&after.revision, result.revision());
            assert_block_resolves(base, &before.address);
            assert_block_resolves(result, &after.address);
        }
        PatchReceipt::InsertBlock { outcome } => {
            let InsertBlockOutcome::Inserted { after, .. } = outcome;
            assert_eq!(&after.revision, result.revision());
            assert_block_resolves(result, &after.address);
        }
        PatchReceipt::SetTaskStatus { outcome } => {
            let (before, after) = match outcome {
                SetTaskOutcome::NoChange { before, after }
                | SetTaskOutcome::Replaced { before, after } => (before, after),
            };
            assert_eq!(&before.revision, base.revision());
            assert_eq!(&after.revision, result.revision());
            base.resolve(&TargetAddress::Task {
                block: before.block.clone(),
                path: before.path.clone(),
            })
            .expect("receipt task before identity resolves");
            result
                .resolve(&TargetAddress::Task {
                    block: after.block.clone(),
                    path: after.path.clone(),
                })
                .expect("receipt task after identity resolves");
        }
        PatchReceipt::SetFrontmatter { outcome } => {
            let (before, after) = match outcome {
                SetFrontmatterOutcome::NoChange { before, after }
                | SetFrontmatterOutcome::Inserted { before, after }
                | SetFrontmatterOutcome::Replaced { before, after } => (before, after),
            };
            assert_eq!(&before.revision, base.revision());
            assert_eq!(&after.revision, result.revision());
            base.resolve(&TargetAddress::FrontmatterField {
                path: before.path.clone(),
            })
            .expect("receipt field before identity resolves");
            result
                .resolve(&TargetAddress::FrontmatterField {
                    path: after.path.clone(),
                })
                .expect("receipt field after identity resolves");
        }
        PatchReceipt::ReplaceTableRow { outcome } => {
            let (before, after) = match outcome {
                ReplaceTableRowOutcome::NoChange { before, after }
                | ReplaceTableRowOutcome::Replaced { before, after } => (before, after),
            };
            assert_eq!(&before.revision, base.revision());
            assert_eq!(&after.revision, result.revision());
            base.resolve(&TargetAddress::TableRow {
                table: before.table.clone(),
                row: before.row,
            })
            .expect("receipt row before identity resolves");
            result
                .resolve(&TargetAddress::TableRow {
                    table: after.table.clone(),
                    row: after.row,
                })
                .expect("receipt row after identity resolves");
        }
        other => panic!("unexpected generated receipt: {other:?}"),
    }
}

fn operation_tag(operation: &PatchOp) -> &'static str {
    match operation {
        PatchOp::ReplaceBlock { .. } => "replace_block",
        PatchOp::DeleteBlock { .. } => "delete_block",
        PatchOp::InsertBlock { .. } => "insert_block",
        PatchOp::MoveBlock { .. } => "move_block",
        PatchOp::ReplaceSection { .. } => "replace_section",
        PatchOp::InsertSection { .. } => "insert_section",
        PatchOp::ReplacePreamble { .. } => "replace_preamble",
        PatchOp::DeleteSection { .. } => "delete_section",
        PatchOp::MoveSection { .. } => "move_section",
        PatchOp::SetTaskStatus { .. } => "set_task_status",
        PatchOp::SetFrontmatter { .. } => "set_frontmatter",
        PatchOp::DeleteFrontmatter { .. } => "delete_frontmatter",
        PatchOp::ReplaceTableRow { .. } => "replace_table_row",
        PatchOp::InsertTableRow { .. } => "insert_table_row",
        PatchOp::DeleteTableRow { .. } => "delete_table_row",
    }
}

fn receipt_tag(receipt: &PatchReceipt) -> &'static str {
    match receipt {
        PatchReceipt::ReplaceBlock { .. } => "replace_block",
        PatchReceipt::DeleteBlock { .. } => "delete_block",
        PatchReceipt::InsertBlock { .. } => "insert_block",
        PatchReceipt::MoveBlock { .. } => "move_block",
        PatchReceipt::ReplaceSection { .. } => "replace_section",
        PatchReceipt::InsertSection { .. } => "insert_section",
        PatchReceipt::ReplacePreamble { .. } => "replace_preamble",
        PatchReceipt::DeleteSection { .. } => "delete_section",
        PatchReceipt::MoveSection { .. } => "move_section",
        PatchReceipt::SetTaskStatus { .. } => "set_task_status",
        PatchReceipt::SetFrontmatter { .. } => "set_frontmatter",
        PatchReceipt::DeleteFrontmatter { .. } => "delete_frontmatter",
        PatchReceipt::ReplaceTableRow { .. } => "replace_table_row",
        PatchReceipt::InsertTableRow { .. } => "insert_table_row",
        PatchReceipt::DeleteTableRow { .. } => "delete_table_row",
    }
}

fn operation_matches_receipt(operation: &PatchOp, receipt: &PatchReceipt) -> bool {
    match (operation, receipt) {
        (PatchOp::ReplaceBlock { target, .. }, PatchReceipt::ReplaceBlock { outcome }) => {
            let before = match outcome {
                ReplaceBlockOutcome::NoChange { before, .. }
                | ReplaceBlockOutcome::Replaced { before, .. }
                | ReplaceBlockOutcome::Deleted { before, .. } => before,
            };
            before.address == target.address
        }
        (PatchOp::InsertBlock { .. }, PatchReceipt::InsertBlock { .. }) => true,
        (PatchOp::SetTaskStatus { target, .. }, PatchReceipt::SetTaskStatus { outcome }) => {
            let before = match outcome {
                SetTaskOutcome::NoChange { before, .. }
                | SetTaskOutcome::Replaced { before, .. } => before,
            };
            before.block == target.block && before.path == target.path
        }
        (PatchOp::SetFrontmatter { target, .. }, PatchReceipt::SetFrontmatter { outcome }) => {
            let before = match outcome {
                SetFrontmatterOutcome::NoChange { before, .. }
                | SetFrontmatterOutcome::Inserted { before, .. }
                | SetFrontmatterOutcome::Replaced { before, .. } => before,
            };
            before.path == target.path
        }
        (PatchOp::ReplaceTableRow { target, .. }, PatchReceipt::ReplaceTableRow { outcome }) => {
            let before = match outcome {
                ReplaceTableRowOutcome::NoChange { before, .. }
                | ReplaceTableRowOutcome::Replaced { before, .. } => before,
            };
            before.table == target.table && before.row == target.row
        }
        _ => false,
    }
}

#[test]
fn review_premises_exercise_shifted_receipts_and_sibling_fields() {
    let case = DocumentCase {
        crlf: true,
        final_newline: false,
        frontmatter: true,
        setext_root: true,
        extra_blank_line: true,
        trailing_spaces: true,
        referenced_footnote: false,
        atom: "premise".into(),
    };
    let transaction = TransactionCase {
        operation_mask: 2 | 4 | 8 | 16 | 32,
        insertion_before: true,
        second_task: true,
        second_row: true,
        atom: "changed".into(),
    };
    let document = case.document();
    assert!(document.source().contains("preamble-premise  \r\n\r\n\r\n"));
    assert_generated_inventory(&document, &case);
    let patch = transaction.patch(&document);
    let outcome = patch.apply(&document).unwrap();
    assert_eq!(
        patch
            .operations
            .iter()
            .map(operation_tag)
            .collect::<Vec<_>>(),
        outcome.receipts.iter().map(receipt_tag).collect::<Vec<_>>()
    );

    let task = outcome
        .receipts
        .iter()
        .find_map(|receipt| match receipt {
            PatchReceipt::SetTaskStatus { outcome } => Some(outcome),
            _ => None,
        })
        .unwrap();
    let (task_before, task_after) = match task {
        SetTaskOutcome::NoChange { before, after } | SetTaskOutcome::Replaced { before, after } => {
            (before, after)
        }
    };
    assert_ne!(task_before.block.ordinal, task_after.block.ordinal);
    let tasks = snapshots(&outcome.document, TargetKind::Task)
        .into_iter()
        .map(|snapshot| {
            outcome
                .document
                .resolve(&snapshot.address)
                .unwrap()
                .read_task(&outcome.document)
                .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        tasks
            .iter()
            .find(|task| task.summary.starts_with("child-"))
            .unwrap()
            .status,
        TaskStatus::Done
    );
    assert_eq!(
        tasks
            .iter()
            .find(|task| task.summary.starts_with("parent-"))
            .unwrap()
            .status,
        TaskStatus::Pending
    );

    let row = outcome
        .receipts
        .iter()
        .find_map(|receipt| match receipt {
            PatchReceipt::ReplaceTableRow { outcome } => Some(outcome),
            _ => None,
        })
        .unwrap();
    let (row_before, row_after) = match row {
        ReplaceTableRowOutcome::NoChange { before, after }
        | ReplaceTableRowOutcome::Replaced { before, after } => (before, after),
    };
    assert_ne!(row_before.table.ordinal, row_after.table.ordinal);
    let row_read = outcome
        .document
        .resolve(&TargetAddress::TableRow {
            table: row_after.table.clone(),
            row: row_after.row,
        })
        .unwrap()
        .read_table_row(&outcome.document)
        .unwrap();
    assert_eq!(row_read.cells, vec!["stable-new-changed", "changed"]);

    let k1 = outcome
        .document
        .resolve(&TargetAddress::FrontmatterField {
            path: vec!["k1".into()],
        })
        .unwrap()
        .read_frontmatter_field(&outcome.document)
        .unwrap();
    assert_eq!(
        k1.value,
        Some(serde_json::json!("frontmatter-sibling-changed"))
    );
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: PROPERTY_CASES,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            "tests/transaction_invariants.proptest-regressions",
        ))),
        ..ProptestConfig::default()
    })]

    #[test]
    fn generated_documents_preserve_map_resolve_read_agreement(case in document_case()) {
        let document = case.document();
        assert!(document.source().len() < 8 * 1024, "generated source exceeded budget: {case:?}");
        assert_document_contract(&document);
        assert_generated_inventory(&document, &case);
    }

    #[test]
    fn generated_valid_transactions_replay_and_bind_receipts(
        case in document_case(),
        transaction in transaction_case(),
    ) {
        let rendered = case.render();
        let document = case.parse(rendered.clone());
        let patch = transaction.patch(&document);
        let patch_json = serde_json::to_string_pretty(&patch).expect("generated patch serializes");
        let context = format!(
            "case={case:?}\ntransaction={transaction:?}\nsource={rendered:?}\npatch={patch_json}"
        );
        prop_assert!((1..=6).contains(&patch.operations.len()), "{}", context);

        let outcome = patch
            .apply(&document)
            .unwrap_or_else(|error| panic!("generated valid patch failed: {error}\n{context}"));
        let replay = patch
            .apply(&document)
            .unwrap_or_else(|error| panic!("same-base replay failed: {error}\n{context}"));
        prop_assert_eq!(replay.document.source(), outcome.document.source(), "{}", context);
        prop_assert_eq!(replay.document.revision(), outcome.document.revision(), "{}", context);
        prop_assert_eq!(&replay.receipts, &outcome.receipts, "{}", context);

        let decoded: Patch = serde_json::from_value(
            serde_json::to_value(&patch).expect("generated patch serializes"),
        )
        .expect("generated patch deserializes");
        let decoded_outcome = decoded
            .apply(&document)
            .unwrap_or_else(|error| panic!("wire patch failed: {error}\n{context}"));
        prop_assert_eq!(decoded_outcome.document.source(), outcome.document.source(), "{}", context);
        prop_assert_eq!(decoded_outcome.document.revision(), outcome.document.revision(), "{}", context);
        prop_assert_eq!(&decoded_outcome.receipts, &outcome.receipts, "{}", context);

        prop_assert_eq!(outcome.receipts.len(), patch.operations.len(), "{}", context);
        let operation_tags = patch.operations.iter().map(operation_tag).collect::<Vec<_>>();
        let receipt_tags = outcome.receipts.iter().map(receipt_tag).collect::<Vec<_>>();
        prop_assert_eq!(&receipt_tags, &operation_tags, "receipt order diverged\n{}", context);
        for (operation, receipt) in patch.operations.iter().zip(&outcome.receipts) {
            prop_assert!(
                operation_matches_receipt(operation, receipt),
                "receipt identity diverged from its operation\n{}",
                context
            );
        }
        for receipt in &outcome.receipts {
            let round_trip: PatchReceipt = serde_json::from_value(
                serde_json::to_value(receipt).expect("receipt serializes"),
            )
            .expect("receipt deserializes");
            prop_assert_eq!(&round_trip, receipt, "{}", context);
            let binding = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                assert_receipt_bindings(&document, &outcome.document, receipt)
            }));
            prop_assert!(binding.is_ok(), "receipt binding failed\n{}", context);
        }
        let document_contract = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            assert_document_contract(&outcome.document);
            assert_generated_inventory(&outcome.document, &case);
        }));
        prop_assert!(document_contract.is_ok(), "result document contract failed\n{}", context);
        for sentinel in case.untouched_boundary_sentinels() {
            prop_assert!(
                outcome.document.source().contains(&sentinel),
                "untouched boundary bytes changed: {sentinel:?}\n{}",
                context
            );
        }
        let stable_row = find_snapshot(&outcome.document, |snapshot| {
            matches!(&snapshot.summary, TargetSummary::TableRow { cells, .. } if cells.first().is_some_and(|cell| cell.starts_with("stable")))
        });
        let expected_stable_cells = if transaction.operation_mask & 8 != 0 && transaction.second_row {
            vec![format!("stable-new-{}", transaction.atom), "changed".into()]
        } else {
            vec![format!("stable-{}", case.atom), "fixed".into()]
        };
        prop_assert!(
            matches!(stable_row.summary, TargetSummary::TableRow { ref cells, .. } if cells == &expected_stable_cells),
            "stable table row diverged\n{}",
            context
        );

        let k1 = outcome
            .document
            .resolve(&TargetAddress::FrontmatterField { path: vec!["k1".into()] })
            .expect("result k1 remains addressable")
            .read_frontmatter_field(&outcome.document)
            .expect("result k1 reads")
            .value;
        let expected_k1 = if transaction.operation_mask & 32 != 0 {
            Some(serde_json::json!(format!("frontmatter-sibling-{}", transaction.atom)))
        } else if case.frontmatter {
            Some(serde_json::json!("stable"))
        } else {
            None
        };
        prop_assert_eq!(k1, expected_k1, "sibling frontmatter field diverged\n{}", context);

        let task_reads = snapshots(&outcome.document, TargetKind::Task)
            .into_iter()
            .map(|snapshot| {
                outcome
                    .document
                    .resolve(&snapshot.address)
                    .expect("result task resolves")
                    .read_task(&outcome.document)
                    .expect("result task reads")
            })
            .collect::<Vec<_>>();
        let parent = task_reads
            .iter()
            .find(|task| task.summary.starts_with("parent-"))
            .expect("generated parent task remains");
        let child = task_reads
            .iter()
            .find(|task| task.summary.starts_with("child-"))
            .expect("generated child task remains");
        let (expected_parent, expected_child) = if transaction.operation_mask & 2 == 0 {
            (TaskStatus::Pending, TaskStatus::Pending)
        } else if transaction.second_task {
            (TaskStatus::Pending, TaskStatus::Done)
        } else {
            (TaskStatus::Done, TaskStatus::Pending)
        };
        prop_assert_eq!(parent.status, expected_parent, "parent task changed unexpectedly\n{}", context);
        prop_assert_eq!(child.status, expected_child, "child task changed unexpectedly\n{}", context);

        let mut reversed_patch = patch.clone();
        reversed_patch.operations.reverse();
        let reversed = reversed_patch
            .apply(&document)
            .unwrap_or_else(|error| panic!("disjoint permutation failed: {error}\n{context}"));
        let reversed_tags = reversed.receipts.iter().map(receipt_tag).collect::<Vec<_>>();
        let expected_reversed_tags = operation_tags.iter().rev().copied().collect::<Vec<_>>();
        prop_assert_eq!(&reversed_tags, &expected_reversed_tags, "reversed receipt order diverged\n{}", context);
        for (operation, receipt) in reversed_patch.operations.iter().zip(&reversed.receipts) {
            prop_assert!(
                operation_matches_receipt(operation, receipt),
                "reversed receipt identity diverged from its operation\n{}",
                context
            );
        }
        for sentinel in case.untouched_boundary_sentinels() {
            prop_assert!(
                reversed.document.source().contains(&sentinel),
                "reversed patch changed untouched boundary bytes: {sentinel:?}\n{}",
                context
            );
        }
        let inserts_two_missing_frontmatter_fields = !case.frontmatter
            && transaction.operation_mask & 4 != 0
            && transaction.operation_mask & 32 != 0;
        if !inserts_two_missing_frontmatter_fields {
            prop_assert_eq!(reversed.document.source(), outcome.document.source(), "{}", context);
            prop_assert_eq!(reversed.document.revision(), outcome.document.revision(), "{}", context);
            let expected_reversed_receipts = outcome.receipts.iter().rev().cloned().collect::<Vec<_>>();
            prop_assert_eq!(reversed.receipts, expected_reversed_receipts, "reversed receipts differ beyond order\n{}", context);
        }

        prop_assert!(matches!(
            patch.apply(&outcome.document),
            Err(mdtools::core_error::CoreError::DocumentRevisionMismatch { .. })
        ), "changed result must reject a patch bound to the base revision\n{}", context);
    }

    #[test]
    fn generated_nochange_replacements_are_byte_identical(case in document_case()) {
        let document = case.document();
        let block = find_snapshot(&document, |snapshot| {
            matches!(&snapshot.summary, TargetSummary::Block { preview, .. } if preview.starts_with("paragraph-main-"))
        });
        let markdown = document
            .slice(&block.selection_span.expect("body block selection"))
            .expect("body block slices")
            .to_string();
        let patch = Patch {
            base_revision: document.revision().clone(),
            operations: vec![PatchOp::ReplaceBlock {
                target: ReplaceBlockTarget::try_from(&block).expect("block evidence refines"),
                markdown,
            }],
        };
        let outcome = patch.apply(&document).expect("no-change replacement applies");
        prop_assert_eq!(outcome.document.source(), document.source());
        prop_assert_eq!(outcome.document.revision(), document.revision());
        prop_assert_eq!(outcome.receipts[0].disposition(), MutationDisposition::NoChange);
        assert_receipt_bindings(&document, &outcome.document, &outcome.receipts[0]);
    }

    #[test]
    fn generated_nested_target_conflicts_are_atomic(case in document_case()) {
        let rendered = case.render();
        let document = case.parse(rendered.clone());
        let source_before = document.source().to_string();
        let revision_before = document.revision().clone();
        let map_before = document.map().expect("base maps before refusal");
        let list = find_snapshot(&document, |snapshot| {
            matches!(snapshot.summary, TargetSummary::Block { kind: BlockKind::List, .. })
        });
        let task = find_snapshot(&document, |snapshot| snapshot.kind == TargetKind::Task);
        let unchanged_list = document
            .slice(&list.selection_span.expect("list selection"))
            .expect("list slices")
            .to_string();
        let patch = Patch {
            base_revision: document.revision().clone(),
            operations: vec![
                PatchOp::ReplaceBlock {
                    target: ReplaceBlockTarget::try_from(&list).expect("list evidence refines"),
                    markdown: unchanged_list,
                },
                PatchOp::SetTaskStatus {
                    target: TaskPatchTarget::try_from(&task).expect("task evidence refines"),
                    status: TaskStatus::Done,
                },
            ],
        };
        let patch_json = serde_json::to_string_pretty(&patch).expect("conflict patch serializes");
        let context = format!("case={case:?}\nsource={rendered:?}\npatch={patch_json}");

        prop_assert!(matches!(
            patch.apply(&document),
            Err(mdtools::core_error::CoreError::PatchInvariant(message))
                if message.contains("overlap")
        ), "owning block and nested task must conflict atomically\n{}", context);
        // Document is immutable today; these assertions are deliberate tripwires
        // against future interior caches or a mutable source ledger.
        prop_assert_eq!(document.source(), source_before, "{}", context);
        prop_assert_eq!(document.revision(), &revision_before, "{}", context);
        prop_assert_eq!(
            document.map().expect("base remaps after refusal"),
            map_before,
            "{}",
            context
        );
        assert_document_contract(&document);
    }

    #[test]
    fn generated_stale_transactions_refuse_before_other_work(
        case in document_case(),
        transaction in transaction_case(),
    ) {
        let rendered = case.render();
        let document = case.parse(rendered.clone());
        let mut stale_transaction = transaction.clone();
        stale_transaction.operation_mask |= 1;
        let patch = stale_transaction.patch(&document);
        let patch_json = serde_json::to_string_pretty(&patch).expect("stale patch serializes");
        let context = format!(
            "case={case:?}\ntransaction={stale_transaction:?}\nsource={rendered:?}\npatch={patch_json}"
        );
        let current_source = if case.crlf {
            "current-only\r\n".to_string()
        } else {
            "current-only\n".to_string()
        };
        let current = case.parse(current_source);
        let source_before = current.source().to_string();
        let revision_before = current.revision().clone();
        let map_before = current.map().expect("current maps before stale refusal");

        prop_assert!(matches!(
            patch.apply(&current),
            Err(mdtools::core_error::CoreError::DocumentRevisionMismatch { .. })
        ), "stale base revision must precede missing-target validation\n{}", context);
        // These documentary checks become behaviorally active if Document ever
        // gains interior mutation during failed planning.
        prop_assert_eq!(current.source(), source_before, "{}", context);
        prop_assert_eq!(current.revision(), &revision_before, "{}", context);
        prop_assert_eq!(
            current.map().expect("current remaps after stale refusal"),
            map_before,
            "{}",
            context
        );
        assert_document_contract(&current);
    }
}
