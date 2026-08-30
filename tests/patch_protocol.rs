use mdtools::core_error::CoreError;
use mdtools::document::Document;
use mdtools::fragment::SectionFragment;
use mdtools::patch::{
    BlockInsertionTarget, DocumentEdge, FrontmatterFieldIdentity, FrontmatterPatchTarget,
    HeadingPatchTarget, HeadingSectionIdentity, Patch, PatchOp, PatchReceipt, PreamblePatchTarget,
    ReplaceBlockTarget, SectionInsertionTarget, SectionPatchTarget, TaskIdentity, TaskPatchTarget,
};
use mdtools::target::{GuardAuthority, TargetAddress, TargetSnapshot, TargetSummary};
use mdtools::{BlockKind, MutationDisposition};

fn paragraph_snapshot(document: &Document) -> TargetSnapshot {
    document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| {
            matches!(
                snapshot.summary,
                TargetSummary::Block {
                    kind: BlockKind::Paragraph,
                    ..
                }
            )
        })
        .unwrap()
}

fn section_snapshot(document: &Document, heading: &str) -> TargetSnapshot {
    document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| {
            matches!(&snapshot.summary, TargetSummary::Section { heading: value, .. } if value == heading)
        })
        .unwrap()
}

fn replace_patch(document: &Document, markdown: &str) -> Patch {
    Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::ReplaceBlock {
            target: ReplaceBlockTarget::try_from(&paragraph_snapshot(document)).unwrap(),
            markdown: markdown.into(),
        }],
    }
}

#[test]
fn serialized_patch_executes_guarded_replacement_and_returns_receipt() {
    let document = Document::parse("# Title\n\nbefore\n").unwrap();
    let patch = replace_patch(&document, "after");
    let wire = serde_json::to_string(&patch).unwrap();
    let decoded: Patch = serde_json::from_str(&wire).unwrap();
    assert_eq!(decoded, patch);

    let outcome = decoded.apply(&document).unwrap();
    assert_eq!(outcome.document.source(), "# Title\n\nafter\n");
    assert_eq!(outcome.receipts.len(), 1);
    let receipt = &outcome.receipts[0];
    assert_eq!(receipt.disposition(), MutationDisposition::Replaced);
    assert_eq!(
        receipt.replace_block_before().unwrap().revision,
        *document.revision()
    );
    assert_eq!(
        receipt.replace_block_after().unwrap().revision,
        *outcome.document.revision()
    );
    let TargetAddress::Block { block } = paragraph_snapshot(&document).address else {
        unreachable!()
    };
    assert_eq!(&receipt.replace_block_before().unwrap().address, &block);
}

#[test]
fn unchanged_replacement_is_byte_identical_and_round_trips_receipt() {
    let document = Document::parse("# Title\n\nbefore\n").unwrap();
    let patch = replace_patch(&document, "before\n");
    let outcome = patch.apply(&document).unwrap();
    assert_eq!(outcome.document.source(), document.source());
    let receipt = &outcome.receipts[0];
    assert_eq!(receipt.disposition(), MutationDisposition::NoChange);
    assert_eq!(
        receipt.replace_block_before().unwrap(),
        receipt.replace_block_after().unwrap()
    );
    let wire = serde_json::to_value(receipt).unwrap();
    assert_eq!(
        serde_json::from_value::<PatchReceipt>(wire).unwrap(),
        *receipt
    );
}

#[test]
fn new_u5_wire_shapes_round_trip_and_execute() {
    for fragment in [
        SectionFragment::Semantic {
            markdown: "# Child".into(),
        },
        SectionFragment::Literal {
            markdown: "\n\n## Child".into(),
        },
    ] {
        let wire = serde_json::to_value(&fragment).unwrap();
        assert_eq!(
            serde_json::from_value::<SectionFragment>(wire).unwrap(),
            fragment
        );
    }

    let document = Document::parse("# Parent\n\nbody").unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::InsertSection {
            target: SectionInsertionTarget::try_from(&section_snapshot(&document, "Parent"))
                .unwrap(),
            fragment: SectionFragment::Semantic {
                markdown: "# Child".into(),
            },
        }],
    };
    let decoded: Patch = serde_json::from_value(serde_json::to_value(&patch).unwrap()).unwrap();
    let outcome = decoded.apply(&document).unwrap();
    assert!(outcome.document.source().contains("## Child"));
    let receipt = serde_json::to_value(&outcome.receipts[0]).unwrap();
    assert_eq!(
        serde_json::from_value::<PatchReceipt>(receipt).unwrap(),
        outcome.receipts[0]
    );

    let document = Document::parse("lead\n\n# H\n").unwrap();
    let preamble = document.resolve(&TargetAddress::Preamble).unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::ReplacePreamble {
            target: PreamblePatchTarget::try_from(preamble.snapshot()).unwrap(),
            markdown: "new lead".into(),
        }],
    };
    let decoded: Patch = serde_json::from_value(serde_json::to_value(&patch).unwrap()).unwrap();
    assert_eq!(
        decoded.apply(&document).unwrap().document.source(),
        "new lead\n\n# H\n"
    );
}

#[test]
fn patch_evidence_and_identity_paths_reject_empty_wire_values() {
    let document = Document::parse("# H\n\n- [ ] task\n").unwrap();
    let snapshots = document.map().unwrap();
    let section = snapshots
        .iter()
        .find(|snapshot| snapshot.kind == mdtools::target::TargetKind::Section)
        .unwrap();
    let task = snapshots
        .iter()
        .find(|snapshot| snapshot.kind == mdtools::target::TargetKind::Task)
        .unwrap();

    let mut heading = serde_json::to_value(HeadingPatchTarget::try_from(section).unwrap()).unwrap();
    heading["path"] = serde_json::json!([]);
    assert!(serde_json::from_value::<HeadingPatchTarget>(heading).is_err());

    let mut zero_heading =
        serde_json::to_value(HeadingPatchTarget::try_from(section).unwrap()).unwrap();
    zero_heading["path"][0]["occurrence"] = serde_json::json!(0);
    assert!(serde_json::from_value::<HeadingPatchTarget>(zero_heading).is_err());

    let mut zero_receipt_identity =
        serde_json::to_value(HeadingSectionIdentity::try_from(section).unwrap()).unwrap();
    zero_receipt_identity["path"][0]["occurrence"] = serde_json::json!(0);
    assert!(serde_json::from_value::<HeadingSectionIdentity>(zero_receipt_identity).is_err());

    let mut delete = serde_json::to_value(SectionPatchTarget::try_from(section).unwrap()).unwrap();
    delete["address"]["path"] = serde_json::json!([]);
    assert!(serde_json::from_value::<SectionPatchTarget>(delete).is_err());

    let mut task_target = serde_json::to_value(TaskPatchTarget::try_from(task).unwrap()).unwrap();
    task_target["path"] = serde_json::json!([]);
    assert!(serde_json::from_value::<TaskPatchTarget>(task_target).is_err());

    let mut task_identity = serde_json::to_value(TaskIdentity::try_from(task).unwrap()).unwrap();
    task_identity["path"] = serde_json::json!([]);
    assert!(serde_json::from_value::<TaskIdentity>(task_identity).is_err());

    let document = Document::parse_for_frontmatter_mutation("---\na: old\n---\n\nbody\n").unwrap();
    let field = document
        .resolve(&TargetAddress::FrontmatterField {
            path: vec!["a".into()],
        })
        .unwrap();
    let mut field_target =
        serde_json::to_value(FrontmatterPatchTarget::try_from(field.snapshot()).unwrap()).unwrap();
    field_target["path"] = serde_json::json!([]);
    assert!(serde_json::from_value::<FrontmatterPatchTarget>(field_target).is_err());

    let mut field_identity =
        serde_json::to_value(FrontmatterFieldIdentity::try_from(field.snapshot()).unwrap())
            .unwrap();
    field_identity["path"] = serde_json::json!([]);
    assert!(serde_json::from_value::<FrontmatterFieldIdentity>(field_identity).is_err());

    for mode in ["semantic", "literal"] {
        assert!(
            serde_json::from_value::<SectionFragment>(serde_json::json!({
                "mode": mode,
                "markdown": ""
            }))
            .is_err()
        );
    }
}

#[test]
fn patch_decoder_enforces_schema_cardinality_and_insert_payloads() {
    let document = Document::parse("body\n").unwrap();
    assert!(serde_json::from_value::<Patch>(serde_json::json!({
        "base_revision": document.revision(),
        "operations": []
    }))
    .is_err());

    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::InsertBlock {
            target: BlockInsertionTarget::DocumentEdge {
                edge: DocumentEdge::End,
                revision: document.revision().clone(),
            },
            markdown: "inserted".into(),
        }],
    };
    let mut wire = serde_json::to_value(patch).unwrap();
    wire["operations"][0]["markdown"] = serde_json::json!("");
    assert!(serde_json::from_value::<Patch>(wire).is_err());
}

#[test]
fn move_section_receipts_reject_preamble_identities() {
    let revision = Document::parse("# H\n").unwrap().revision().clone();
    let preamble = serde_json::json!({
        "address": { "kind": "preamble" },
        "revision": revision
    });
    let wire = serde_json::json!({
        "operation": "move_section",
        "destination_before": preamble,
        "outcome": {
            "disposition": "no_change",
            "before": preamble,
            "after": preamble
        }
    });
    assert!(serde_json::from_value::<PatchReceipt>(wire).is_err());

    let schema = mdtools::protocol::patch_receipt_schema();
    let rendered = serde_json::to_string(&schema["$defs"]["MoveSectionOutcome"]).unwrap();
    assert!(rendered.contains("HeadingSectionIdentity"));
    assert!(!rendered.contains("#/$defs/SectionIdentity\""));
}

#[test]
fn stale_patch_changes_nothing() {
    let original = Document::parse("before\n").unwrap();
    let current = Document::parse("current\n").unwrap();
    let patch = Patch {
        base_revision: original.revision().clone(),
        operations: Vec::new(),
    };
    assert!(matches!(
        patch.apply(&current),
        Err(CoreError::DocumentRevisionMismatch { .. })
    ));
}

#[test]
fn stale_target_changes_nothing() {
    let document = Document::parse("before\n").unwrap();
    let mut stale = ReplaceBlockTarget::try_from(&paragraph_snapshot(&document)).unwrap();
    stale.guard.etag = mdtools::fingerprint::TargetEtag::for_bytes(b"stale");
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::ReplaceBlock {
            target: stale,
            markdown: "# becomes a heading\n\nsecond block".into(),
        }],
    };
    assert!(matches!(
        patch.apply(&document),
        Err(CoreError::TargetAuthorityMismatch { .. })
    ));
    assert_eq!(document.source(), "before\n");
}

#[test]
fn serialized_snapshot_re_resolves_without_trusting_index_identity() {
    let source = "---\ntitle: [\n---\n# Heading\n\nbody\n";
    let lenient = Document::parse(source).unwrap();
    let strict = Document::parse_for_frontmatter(source).unwrap();
    let snapshot = paragraph_snapshot(&lenient);
    let current = strict
        .resolve(&snapshot.address)
        .unwrap()
        .snapshot()
        .clone();
    assert_eq!(
        current, snapshot,
        "transport evidence matches current target"
    );
    let evidence = ReplaceBlockTarget::try_from(&snapshot).unwrap();
    let patch = Patch {
        base_revision: strict.revision().clone(),
        operations: vec![PatchOp::ReplaceBlock {
            target: evidence,
            markdown: "changed".into(),
        }],
    };
    let patch: Patch = serde_json::from_value(serde_json::to_value(patch).unwrap()).unwrap();
    let result = std::panic::catch_unwind(|| patch.apply(&strict));
    assert!(
        result.is_ok(),
        "serialized evidence must not carry an index handle"
    );
    let outcome = result.unwrap().unwrap();
    assert_eq!(
        outcome.receipts[0].replace_block_before().unwrap(),
        &mdtools::patch::ReplaceBlockState::try_from(&current).unwrap()
    );
    assert!(outcome.document.source().contains("changed"));
}

#[test]
fn serialized_evidence_for_an_address_absent_from_current_index_is_rejected() {
    let source = "---\ntitle: [\n---\n# Heading\n\nbody\n";
    let lenient = Document::parse(source).unwrap();
    let strict = Document::parse_for_frontmatter(source).unwrap();
    let snapshot = lenient
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| {
            snapshot.kind == mdtools::target::TargetKind::Block
                && matches!(
                    strict.resolve(&snapshot.address),
                    Err(CoreError::TargetNotFound { .. })
                )
        })
        .expect("lenient parse has a block absent from strict index");
    let evidence = ReplaceBlockTarget::try_from(&snapshot).unwrap();
    let patch = Patch {
        base_revision: strict.revision().clone(),
        operations: vec![PatchOp::ReplaceBlock {
            target: evidence,
            markdown: "changed".into(),
        }],
    };
    assert!(matches!(
        patch.apply(&strict),
        Err(CoreError::TargetNotFound { .. })
    ));
}

#[test]
fn malformed_addresses_and_unknown_protocol_fields_are_rejected() {
    let document = Document::parse("body\n").unwrap();
    let patch = replace_patch(&document, "changed");
    let mut wire = serde_json::to_value(patch).unwrap();
    wire["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Patch>(wire).is_err());

    let patch = replace_patch(&document, "changed");
    let mut wire = serde_json::to_value(patch).unwrap();
    wire["operations"][0]["target"]["address"]["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Patch>(wire).is_err());

    let invalid = TargetAddress::Task {
        block: mdtools::target::BlockAddress {
            section: mdtools::target::SectionAddress::Preamble,
            ordinal: 0,
        },
        path: Vec::new(),
    };
    assert!(
        serde_json::from_value::<TargetAddress>(serde_json::to_value(invalid).unwrap()).is_err()
    );
}

#[test]
fn target_guards_and_revisions_are_full_sha256() {
    let document = Document::parse("body\n").unwrap();
    let snapshot = paragraph_snapshot(&document);
    assert_eq!(document.revision().as_str().len(), 64);
    let GuardAuthority::Selection { etag, .. } = snapshot.guard else {
        panic!("paragraph has selection guard")
    };
    assert_eq!(etag.as_str().len(), 64);
}

#[test]
fn invalid_candidate_never_escapes() {
    let document = Document::parse("before\n").unwrap();
    let patch = replace_patch(&document, "# heading\n\nsecond block");
    assert!(matches!(
        patch.apply(&document),
        Err(CoreError::InvalidPatch(_))
    ));
    assert_eq!(document.source(), "before\n");
}

#[test]
fn candidate_reparse_preserves_each_document_parse_policy() {
    let source = "---\ntitle: [\n---\n# Heading\n\nbody\n";
    let lenient = Document::parse(source).unwrap();
    let evidence = paragraph_snapshot(&lenient);
    let target = ReplaceBlockTarget::try_from(&evidence).unwrap();

    let patch_for = |document: &Document| Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::ReplaceBlock {
            target: target.clone(),
            markdown: "changed".into(),
        }],
    };

    let lenient_outcome = patch_for(&lenient).apply(&lenient).unwrap();
    assert!(!lenient.has_frontmatter());
    assert!(!lenient_outcome.document.has_frontmatter());

    let strict = Document::parse_for_frontmatter(source).unwrap();
    let strict_outcome = patch_for(&strict).apply(&strict).unwrap();
    assert!(strict.has_frontmatter());
    assert!(strict_outcome.document.has_frontmatter());

    let mutation = Document::parse_for_frontmatter_mutation(source).unwrap();
    let mutation_outcome = patch_for(&mutation).apply(&mutation).unwrap();
    assert!(mutation.has_frontmatter());
    assert!(mutation_outcome.document.has_frontmatter());
}

#[test]
fn deleting_first_block_emits_no_after_even_when_address_is_reused() {
    let document = Document::parse("one\n\ntwo\n").unwrap();
    let first = paragraph_snapshot(&document);
    let TargetAddress::Block { block } = &first.address else {
        unreachable!()
    };
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::ReplaceBlock {
            target: ReplaceBlockTarget::try_from(&first).unwrap(),
            markdown: String::new(),
        }],
    };
    let outcome = patch.apply(&document).unwrap();
    let receipt = &outcome.receipts[0];
    assert_eq!(receipt.disposition(), MutationDisposition::Deleted);
    assert!(receipt.replace_block_after().is_none());
    assert!(outcome
        .document
        .resolve(&TargetAddress::Block {
            block: block.clone(),
        })
        .is_ok());
}

#[test]
fn patch_rejects_zero_operations_and_accepts_multiple_operations() {
    let document = Document::parse("body\n").unwrap();
    let empty = Patch {
        base_revision: document.revision().clone(),
        operations: Vec::new(),
    };
    assert!(matches!(
        empty.apply(&document),
        Err(CoreError::InvalidPatch(_))
    ));

    let first = paragraph_snapshot(&document);
    let multiple = Patch {
        base_revision: document.revision().clone(),
        operations: vec![
            PatchOp::ReplaceBlock {
                target: ReplaceBlockTarget::try_from(&first).unwrap(),
                markdown: "body\n".into(),
            },
            PatchOp::InsertBlock {
                target: mdtools::patch::BlockInsertionTarget::DocumentEdge {
                    edge: mdtools::patch::DocumentEdge::End,
                    revision: document.revision().clone(),
                },
                markdown: "second".into(),
            },
        ],
    };
    assert_eq!(multiple.apply(&document).unwrap().receipts.len(), 2);
}

#[test]
fn generated_schema_is_strict_and_covers_patch_and_receipt() {
    let schema = mdtools::protocol::protocol_schema();
    let rendered = serde_json::to_string_pretty(&schema).unwrap();
    assert!(rendered.contains("https://json-schema.org/draft/2020-12/schema"));
    assert!(rendered.contains("replace_block"));
    assert!(rendered.contains("PatchReceipt"));
    assert!(rendered.contains("additionalProperties"));
    assert!(rendered.contains("minLength"));
    assert!(rendered.contains("maxLength"));
    assert!(rendered.contains("^[0-9a-f]{64}$"));
    let patch_rendered = serde_json::to_string(&schema["patch"]).unwrap();
    assert!(patch_rendered.contains("ReplaceBlockTarget"));
    assert!(patch_rendered.contains("SelectionGuard"));
    assert!(!patch_rendered.contains("TargetSnapshot"));
    assert!(!patch_rendered.contains("TargetSummary"));
    assert!(!patch_rendered.contains("GuardAuthority"));
    assert!(patch_rendered.contains("HeadingPatchTarget"));
    let move_section = schema["patch"]["$defs"]["PatchOp"]["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .find(|variant| variant["properties"]["op"]["const"] == "move_section")
        .unwrap();
    assert_eq!(
        move_section["properties"]["source"]["$ref"],
        "#/$defs/HeadingPatchTarget"
    );
    assert_eq!(
        move_section["properties"]["destination"]["$ref"],
        "#/$defs/HeadingPatchTarget"
    );
    let receipt_rendered = serde_json::to_string(&schema["patch_receipt"]).unwrap();
    assert!(receipt_rendered.contains("ReplaceBlockOutcome"));
    assert!(receipt_rendered.contains("BlockIdentity"));
    assert!(receipt_rendered.contains("TaskIdentity"));
    assert!(receipt_rendered.contains("BlockInsertionEvidence"));
    assert!(!receipt_rendered.contains("BlockInsertionAddress"));
    assert!(
        !serde_json::to_string(&schema["patch_receipt"]["$defs"]["ReplaceBlockOutcome"])
            .unwrap()
            .contains("inserted")
    );
    assert!(!receipt_rendered.contains("base_revision"));
    assert_eq!(schema["patch"]["properties"]["operations"]["minItems"], 1);
    assert!(schema["patch"]["properties"]["operations"]
        .get("maxItems")
        .is_none());
    assert_eq!(
        schema["patch"]["$defs"]["HeadingAddressSegment"]["properties"]["occurrence"]["minimum"],
        1
    );
    let section_variants = schema["patch"]["$defs"]["SectionAddress"]["oneOf"]
        .as_array()
        .unwrap();
    let heading = section_variants
        .iter()
        .find(|variant| variant["properties"]["kind"]["const"] == "heading")
        .unwrap();
    assert_eq!(heading["properties"]["path"]["minItems"], 1);

    for operation in ["replace_table_row", "insert_table_row"] {
        let variant = schema["patch"]["$defs"]["PatchOp"]["oneOf"]
            .as_array()
            .unwrap()
            .iter()
            .find(|variant| variant["properties"]["op"]["const"] == operation)
            .unwrap();
        assert_eq!(variant["properties"]["markdown"]["minLength"], 1);
    }
}

#[test]
fn receipt_decoder_rejects_impossible_outcomes() {
    let document = Document::parse("before\n").unwrap();
    let receipt = replace_patch(&document, "after")
        .apply(&document)
        .unwrap()
        .receipts
        .remove(0);
    let mut wire = serde_json::to_value(&receipt).unwrap();
    wire["outcome"]["disposition"] = serde_json::json!("inserted");
    assert!(serde_json::from_value::<PatchReceipt>(wire).is_err());

    let mut wire = serde_json::to_value(&receipt).unwrap();
    wire["outcome"].as_object_mut().unwrap().remove("after");
    assert!(serde_json::from_value::<PatchReceipt>(wire).is_err());

    let rendered = serde_json::to_string(&receipt).unwrap();
    assert!(!rendered.contains("base_revision"));
    assert!(!rendered.contains("result_revision"));
    assert_eq!(rendered.matches("\"address\"").count(), 2);
    assert!(!rendered.contains("selection_span"));
    assert_eq!(rendered.matches("\"span\"").count(), 2);
}

#[test]
fn operation_receipts_reject_dispositions_the_operation_cannot_emit() {
    let document = Document::parse("body\n").unwrap();
    let target = ReplaceBlockTarget::try_from(&paragraph_snapshot(&document)).unwrap();
    let patch = Patch {
        base_revision: document.revision().clone(),
        operations: vec![PatchOp::DeleteBlock { target }],
    };
    let receipt = patch.apply(&document).unwrap().receipts.remove(0);
    let mut wire = serde_json::to_value(receipt).unwrap();
    wire["disposition"] = serde_json::json!("inserted");
    assert!(serde_json::from_value::<PatchReceipt>(wire).is_err());

    let receipt_schema = mdtools::protocol::patch_receipt_schema();
    let delete_block = receipt_schema["oneOf"]
        .as_array()
        .unwrap()
        .iter()
        .find(|variant| variant["properties"]["operation"]["const"] == "delete_block")
        .unwrap();
    assert!(delete_block["properties"].get("disposition").is_none());
    assert_eq!(
        delete_block["properties"]["before"]["$ref"],
        "#/$defs/BlockIdentity"
    );
}

#[test]
fn representative_schema_and_receipt_are_reviewable() {
    let document = Document::parse("# Title\n\nbefore\n").unwrap();
    let outcome = replace_patch(&document, "after").apply(&document).unwrap();
    println!(
        "=== patch schema ===\n{}",
        serde_json::to_string_pretty(&mdtools::protocol::patch_schema()).unwrap()
    );
    println!(
        "=== receipt ===\n{}",
        serde_json::to_string_pretty(&outcome.receipts[0]).unwrap()
    );
}
