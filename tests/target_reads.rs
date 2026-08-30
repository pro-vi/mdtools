use mdtools::core_error::CoreError;
use mdtools::document::Document;
use mdtools::read::TargetRead;
use mdtools::target::{JsonValueKind, TargetAddress, TargetKind, TargetSummary};
use mdtools::{BlockKind, MutationDisposition};

const SOURCE: &str = "---\ntitle: Demo\n---\n\nlead\n\n# Work\n\nRead [guide](guide.md).\n\n- [x] finished\n\n| Name | State |\n| --- | --- |\n| A | open |\n";

#[test]
fn map_resolve_and_typed_read_share_one_snapshot_for_every_target() {
    let document = Document::parse_for_frontmatter(SOURCE).unwrap();
    for mapped in document.map().unwrap() {
        let resolved = document.resolve(&mapped.address).unwrap();
        let read = document.read_target(&resolved).unwrap();
        assert_eq!(resolved.snapshot(), &mapped);
        assert_eq!(read.snapshot(), &mapped);
    }
}

#[test]
fn semantic_targets_return_distinct_typed_values() {
    let document = Document::parse_for_frontmatter(SOURCE).unwrap();
    let map = document.map().unwrap();

    let section = map
        .iter()
        .find(|snapshot| snapshot.kind == TargetKind::Section)
        .unwrap();
    let section = document
        .resolve(&section.address)
        .unwrap()
        .read_section(&document)
        .unwrap();
    assert_eq!(section.heading, "Work");
    assert!(section.markdown.starts_with("# Work"));

    let task = map
        .iter()
        .find(|snapshot| snapshot.kind == TargetKind::Task)
        .unwrap();
    let task = document
        .resolve(&task.address)
        .unwrap()
        .read_task(&document)
        .unwrap();
    assert_eq!(task.summary, "finished");
    assert!(task.markdown.contains("[x]"));

    let table = map
        .iter()
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
    let table = document
        .resolve(&table.address)
        .unwrap()
        .read_table(&document)
        .unwrap();
    assert_eq!(table.headers, vec!["Name", "State"]);
    assert_eq!(table.rows, vec![vec!["A", "open"]]);

    let row = map
        .iter()
        .find(|snapshot| snapshot.kind == TargetKind::TableRow)
        .unwrap();
    let row = document
        .resolve(&row.address)
        .unwrap()
        .read_table_row(&document)
        .unwrap();
    assert_eq!(row.cells, vec!["A", "open"]);
    assert_eq!(row.markdown, "| A | open |");

    let link = map
        .iter()
        .find(|snapshot| snapshot.kind == TargetKind::Link)
        .unwrap();
    let link = document
        .resolve(&link.address)
        .unwrap()
        .read_link(&document)
        .unwrap();
    assert_eq!(link.destination.as_deref(), Some("guide.md"));
    assert_eq!(link.markdown, "[guide](guide.md)");
}

#[test]
fn document_read_carries_the_retained_stats_view() {
    let document = Document::parse("# One\n\nbody words\n").unwrap();
    let read = document
        .resolve(&TargetAddress::Document)
        .unwrap()
        .read_document(&document)
        .unwrap();
    assert_eq!(read.stats.heading_count, 1);
    assert_eq!(read.stats.word_count, 3);
    assert_eq!(read.stats.line_count, document.line_count());
}

#[test]
fn table_read_markdown_is_an_unchanged_block_replacement_payload() {
    let document = Document::parse("| Name | State |\n| --- | --- |\n| A | open |\n").unwrap();
    let table_snapshot = document
        .map()
        .unwrap()
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
    let table = document
        .resolve(&table_snapshot.address)
        .unwrap()
        .read_table(&document)
        .unwrap();
    let outcome = mdtools::patch::Patch {
        base_revision: document.revision().clone(),
        operations: vec![mdtools::patch::PatchOp::ReplaceBlock {
            target: mdtools::patch::ReplaceBlockTarget::try_from(&table_snapshot).unwrap(),
            markdown: table.markdown.clone(),
        }],
    }
    .apply(&document)
    .unwrap();
    assert_eq!(
        outcome.receipts[0].disposition(),
        MutationDisposition::NoChange
    );
    assert_eq!(outcome.document.source(), document.source());
}

#[test]
fn missing_frontmatter_field_is_exactly_addressable_and_reads_as_null() {
    let document = Document::parse_for_frontmatter(SOURCE).unwrap();
    let address = TargetAddress::FrontmatterField {
        path: vec!["missing".into(), "value".into()],
    };
    let resolved = document.resolve(&address).unwrap();
    assert_eq!(resolved.snapshot().kind, TargetKind::FrontmatterField);
    assert!(matches!(
        resolved.snapshot().summary,
        TargetSummary::FrontmatterField {
            value: JsonValueKind::Missing,
            ..
        }
    ));
    let read = resolved.read_frontmatter_field(&document).unwrap();
    assert_eq!(read.value, serde_json::Value::Null);
}

#[test]
fn present_null_frontmatter_value_is_not_reported_as_missing() {
    let document = Document::parse_for_frontmatter("---\nvalue: null\n---\n").unwrap();
    let address = TargetAddress::FrontmatterField {
        path: vec!["value".into()],
    };
    let resolved = document.resolve(&address).unwrap();
    assert!(matches!(
        resolved.snapshot().summary,
        TargetSummary::FrontmatterField {
            value: JsonValueKind::Null,
            ..
        }
    ));
    assert_eq!(
        resolved.read_frontmatter_field(&document).unwrap().value,
        serde_json::Value::Null
    );
}

#[test]
fn resolved_target_rejects_a_different_document_before_reading_changed_bytes() {
    let original = Document::parse("# One\n\nbody\n").unwrap();
    let changed = Document::parse("# One\n\nchanged\n").unwrap();
    let section = original
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| snapshot.kind == TargetKind::Section)
        .unwrap();
    let resolved = original.resolve(&section.address).unwrap();
    assert_ne!(original.revision(), changed.revision());
    assert!(matches!(
        resolved.read_section(&changed),
        Err(CoreError::DocumentIndexMismatch)
    ));
}

#[test]
fn resolved_target_rejects_a_different_index_with_identical_source_bytes() {
    let source = "---\ntitle: [\n---\n# Heading\n";
    let lenient = Document::parse(source).unwrap();
    let strict = Document::parse_for_frontmatter(source).unwrap();
    assert_eq!(lenient.revision(), strict.revision());
    let target = lenient.map().unwrap().into_iter().next().unwrap();
    let resolved = lenient.resolve(&target.address).unwrap();

    let result = std::panic::catch_unwind(|| resolved.read(&strict));
    assert!(result.is_ok(), "cross-index reads must not panic");
    assert!(matches!(
        result.unwrap(),
        Err(CoreError::DocumentIndexMismatch)
    ));
}

#[test]
fn typed_read_rejects_the_wrong_domain() {
    let document = Document::parse("# One\n\nbody\n").unwrap();
    let section = document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| snapshot.kind == TargetKind::Section)
        .unwrap();
    let resolved = document.resolve(&section.address).unwrap();
    assert!(matches!(
        resolved.read_task(&document),
        Err(CoreError::TargetKindMismatch { .. })
    ));
}

#[test]
fn generic_read_is_an_enum_of_typed_values_not_a_string_view() {
    let document = Document::parse("text\n").unwrap();
    let block = document
        .map()
        .unwrap()
        .into_iter()
        .find(|snapshot| snapshot.kind == TargetKind::Block)
        .unwrap();
    let read = document
        .read_target(&document.resolve(&block.address).unwrap())
        .unwrap();
    let TargetRead::Block(block) = read else {
        panic!("paragraph must produce a typed block read")
    };
    assert_eq!(block.kind, BlockKind::Paragraph);
    assert_eq!(block.markdown, "text");
}

#[test]
fn absent_frontmatter_and_empty_preamble_have_typed_reads() {
    let document = Document::parse("# Heading\n").unwrap();
    let map = document.map().unwrap();
    let frontmatter = map
        .iter()
        .find(|snapshot| snapshot.kind == TargetKind::Frontmatter)
        .unwrap();
    let frontmatter = document
        .resolve(&frontmatter.address)
        .unwrap()
        .read_frontmatter(&document)
        .unwrap();
    assert!(!frontmatter.present);
    assert!(frontmatter.raw.is_none());

    let preamble = map
        .iter()
        .find(|snapshot| snapshot.kind == TargetKind::Preamble)
        .unwrap();
    let preamble = document
        .resolve(&preamble.address)
        .unwrap()
        .read_preamble(&document)
        .unwrap();
    assert!(preamble.markdown.is_empty());
}

#[test]
fn lexical_selection_is_separate_from_positional_containment() {
    let source = "lead\n\n\n# First\n";
    let document = Document::parse(source).unwrap();
    let map = document.map().unwrap();
    let preamble = map
        .iter()
        .find(|snapshot| snapshot.kind == TargetKind::Preamble)
        .unwrap();
    assert_eq!(
        preamble.selection_span,
        Some(mdtools::SourceSpan {
            line_start: 1,
            line_end: 1,
            byte_start: 0,
            byte_end: 4,
        })
    );
    assert_eq!(
        document
            .resolve(&preamble.address)
            .unwrap()
            .read_preamble(&document)
            .unwrap()
            .markdown,
        "lead"
    );
    let blank_offset = source.find("\n\n\n").unwrap() as u32 + 2;
    assert!(document
        .locate_targets(blank_offset)
        .unwrap()
        .iter()
        .any(|snapshot| snapshot.address == preamble.address));

    let frontmatter = Document::parse_for_frontmatter("---\na: 1\n---\nbody\n").unwrap();
    for field in frontmatter
        .map()
        .unwrap()
        .into_iter()
        .filter(|snapshot| snapshot.kind == TargetKind::FrontmatterField)
    {
        assert_eq!(field.selection_span, None);
        assert!(matches!(
            field.guard,
            mdtools::target::GuardAuthority::Frontmatter { .. }
        ));
    }
    let missing = frontmatter
        .resolve(&TargetAddress::FrontmatterField {
            path: vec!["missing".into()],
        })
        .unwrap();
    assert_eq!(missing.snapshot().selection_span, None);
}
