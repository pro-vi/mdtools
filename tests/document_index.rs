use std::collections::HashSet;

use mdtools::block;
use mdtools::document::Document;
use mdtools::index::IndexNodeKind;
use mdtools::link;
use mdtools::locate;
use mdtools::model::{BlockKind, HeadingMatchMode};
use mdtools::parser::extract_table_projection;
use mdtools::section::{SectionIndex, SectionTarget};
use mdtools::table::{self, TableEditTarget};
use mdtools::task;

#[test]
fn nested_headings_own_only_their_source_ordered_body_blocks() {
    let source = "lead\n\n# Top\n\ntop body\n\n## Child\n\nchild body\n\n# Peer\n\npeer body\n";
    let document = Document::parse(source).unwrap();
    let tree = document.index().render_tree();

    assert_eq!(document.index().node_count(IndexNodeKind::Section), 3);
    assert_eq!(document.index().node_count(IndexNodeKind::BodyBlock), 4);
    assert!(!tree.contains("body-block ordinal=0 parser-index=1 kind=Heading"));

    let top = tree.find("  section level=1 text=\"Top\"").unwrap();
    let top_body = tree[top..]
        .find("    body-block ordinal=0 parser-index=2 kind=Paragraph")
        .unwrap()
        + top;
    let child = tree[top..]
        .find("    section level=2 text=\"Child\"")
        .unwrap()
        + top;
    let child_body = tree[child..]
        .find("      body-block ordinal=0 parser-index=4 kind=Paragraph")
        .unwrap()
        + child;
    let peer = tree.find("  section level=1 text=\"Peer\"").unwrap();
    assert!(top < top_body && top_body < child && child < child_body && child_body < peer);
}

#[test]
fn footnote_ownership_uses_source_position_not_parser_traversal_order() {
    let document = Document::parse(include_str!("fixtures/footnote_midbody.md")).unwrap();
    assert!(document
        .blocks()
        .windows(2)
        .any(|pair| pair[1].span.byte_start < pair[0].span.byte_start));

    let tree = document.index().render_tree();
    let notes = tree.find("  section level=1 text=\"Notes\"").unwrap();
    let footnote = tree[notes..]
        .find("    body-block ordinal=1 parser-index=6 kind=FootnoteDefinition")
        .unwrap()
        + notes;
    let later = tree[notes..]
        .find("    section level=2 text=\"Later\"")
        .unwrap()
        + notes;
    assert!(notes < footnote && footnote < later);
}

#[test]
fn index_projects_every_current_semantic_domain() {
    let source = "---\ntitle: Demo\n---\n\n# Work\n\n- [ ] read [guide](guide.md)\n\n| Name | State |\n| --- | --- |\n| A | open |\n";
    let document = Document::parse_for_frontmatter(source).unwrap();
    let index = document.index();

    assert_eq!(index.node_count(IndexNodeKind::Frontmatter), 1);
    assert_eq!(
        index.node_count(IndexNodeKind::Section),
        SectionIndex::new(&document).outline().len()
    );
    assert_eq!(
        index.node_count(IndexNodeKind::BodyBlock),
        block::blocks(&document)
            .iter()
            .filter(|block| block.kind != BlockKind::Heading)
            .count()
    );
    assert_eq!(
        index.node_count(IndexNodeKind::TaskItem),
        task::tasks(&document, &Default::default()).unwrap().len()
    );
    assert_eq!(
        index.node_count(IndexNodeKind::TableRow),
        table::tables(&document)
            .unwrap()
            .iter()
            .map(|table| table.row_count as usize)
            .sum::<usize>()
    );
    assert_eq!(
        index.node_count(IndexNodeKind::Link),
        link::links(&document).len()
    );

    let tree = index.render_tree();
    assert!(tree.contains("frontmatter format=yaml"));
    assert!(tree.contains("task path=0 task-index=0 status=pending"));
    assert!(tree.contains("headers=[\"Name\", \"State\"]"));
    assert!(tree.contains("table-row ordinal=0 cells=[\"A\", \"open\"]"));
    assert!(tree.contains("destination=Some(\"guide.md\")"));
}

#[test]
fn cached_table_rows_match_projection_locate_and_edit_for_all_indentation_and_line_endings() {
    for newline in ["\n", "\r\n"] {
        for indentation in 0..=3 {
            let indent = " ".repeat(indentation);
            let source = format!(
                "# Table{newline}{newline}{indent}| h |{newline}{indent}|---|{newline}{indent}| v |{newline}"
            );
            let document = Document::parse(&source).unwrap();
            let block = document
                .blocks()
                .iter()
                .find(|block| block.kind == BlockKind::Table)
                .unwrap();
            let cached = block.table.as_ref().unwrap();
            let current =
                extract_table_projection(document.slice(&block.span).unwrap(), block.span).unwrap();

            assert_eq!(cached.headers, current.headers);
            assert_eq!(cached.alignments, current.alignments);
            assert_eq!(cached.rows.len(), current.rows.len());
            let cached_row = &cached.rows[0];
            let current_row = &current.rows[0];
            assert_eq!(cached_row.cells, current_row.cells);
            assert_eq!(cached_row.span, current_row.span);

            let physical_start = source.rfind(&format!("{indent}| v |")).unwrap() as u32;
            let physical_end = physical_start + indentation as u32 + "| v |".len() as u32;
            assert_eq!(cached_row.span.byte_start, physical_start);
            assert_eq!(cached_row.span.byte_end, physical_end);
            if newline == "\r\n" && indentation == 2 {
                assert_eq!(
                    cached_row.span,
                    mdtools::model::SourceSpan {
                        line_start: 5,
                        line_end: 5,
                        byte_start: 29,
                        byte_end: 36,
                    }
                );
            }

            let located = locate::locate(&document, physical_start + indentation as u32)
                .unwrap()
                .table_row
                .unwrap();
            assert_eq!(located.span, cached_row.span);

            let payload = document.slice(&cached_row.span).unwrap().to_string();
            let edit = table::prepare_replace_row(&document, block.index, 0, None)
                .unwrap()
                .replace(payload)
                .unwrap();
            let TableEditTarget::Row { span, .. } = edit.target else {
                panic!("replacement must target a row")
            };
            assert_eq!(span, cached_row.span);
            assert_eq!(edit.preservation.target_span_before, Some(cached_row.span));
        }
    }
}

#[test]
fn heading_links_belong_to_complete_heading_nodes_for_atx_and_setext() {
    for (source, source_kind) in [
        ("# [Guide](guide.md)\n", "atx"),
        ("[Guide](guide.md)\n=================\n", "setext"),
    ] {
        let tree = Document::parse(source).unwrap().index().render_tree();
        let heading = tree
            .lines()
            .find(|line| line.contains("heading level=1"))
            .expect("complete heading node");
        let marker = tree
            .lines()
            .find(|line| line.contains(&format!("heading-marker level=1 source={source_kind}")))
            .unwrap();
        let link = tree
            .lines()
            .find(|line| line.contains("link occurrence=0"))
            .unwrap();

        let heading_indent = heading.len() - heading.trim_start().len();
        let marker_indent = marker.len() - marker.trim_start().len();
        let link_indent = link.len() - link.trim_start().len();
        assert_eq!(marker_indent, heading_indent + 2);
        assert_eq!(link_indent, heading_indent + 2);
    }
}

#[test]
fn indexed_and_legacy_sections_share_final_line_metadata() {
    for source in ["# First", "# First\n"] {
        let document = Document::parse(source).unwrap();
        let legacy = SectionIndex::new(&document)
            .resolve(&SectionTarget::heading("First", None, HeadingMatchMode::Exact).unwrap())
            .unwrap();
        let heading_span = legacy.heading.as_ref().unwrap().span;
        let expected = format!(
            "section level=1 text=\"First\" heading=0..{} lines={}-{} bytes={}..{}",
            heading_span.byte_end,
            legacy.span.line_start,
            legacy.span.line_end,
            legacy.span.byte_start,
            legacy.span.byte_end
        );
        assert!(document.index().render_tree().contains(&expected));
    }
}

#[test]
fn task_ancestry_uses_the_longest_existing_task_path_prefix() {
    let source = "- [ ] root\n  - ordinary\n    - [ ] grandchild\n";
    let tree = Document::parse(source).unwrap().index().render_tree();
    let root = tree
        .lines()
        .find(|line| line.contains("summary=\"root\""))
        .unwrap();
    let grandchild = tree
        .lines()
        .find(|line| line.contains("summary=\"grandchild\""))
        .unwrap();
    let root_indent = root.len() - root.trim_start().len();
    let grandchild_indent = grandchild.len() - grandchild.trim_start().len();
    assert_eq!(grandchild_indent, root_indent + 2);
}

#[test]
fn sibling_nested_lists_have_unique_round_tripping_task_paths() {
    let source = "- [ ] root\n  - [ ] bullet child\n\n  1. [ ] ordered child\n";
    let document = Document::parse(source).unwrap();
    let tasks = task::tasks(&document, &Default::default()).unwrap();
    assert_eq!(tasks.len(), 3);
    assert_eq!(
        tasks
            .iter()
            .map(|task| task.loc.to_string())
            .collect::<HashSet<_>>()
            .len(),
        tasks.len()
    );
    for expected in tasks {
        let direct = task::task(&document, &expected.loc).unwrap();
        assert_eq!(direct.task.span, expected.span);
        assert_eq!(direct.task.summary_text, expected.summary_text);
    }
}

#[test]
fn legacy_section_membership_uses_index_source_ownership_for_footnotes() {
    let source =
        "# Notes\n\nClaim[^a].\n\n[^a]:\n    - [ ] note task\n\n## Later\n\n- [ ] later task\n";
    let document = Document::parse(source).unwrap();
    let section_index = SectionIndex::new(&document);
    let notes = section_index
        .resolve(&SectionTarget::heading("Notes", None, HeadingMatchMode::Exact).unwrap())
        .unwrap();
    let later = section_index
        .resolve(&SectionTarget::heading("Later", None, HeadingMatchMode::Exact).unwrap())
        .unwrap();
    let all_tasks = task::tasks(&document, &Default::default()).unwrap();
    let note_task = all_tasks
        .iter()
        .find(|task| task.summary_text == "note task")
        .unwrap();
    let later_task = all_tasks
        .iter()
        .find(|task| task.summary_text == "later task")
        .unwrap();

    assert!(notes.block_indices.contains(&note_task.loc.block_index()));
    assert!(!later.block_indices.contains(&note_task.loc.block_index()));
    assert!(later.block_indices.contains(&later_task.loc.block_index()));
    assert_eq!(note_task.nearest_heading.as_deref(), Some("Notes"));
    assert_eq!(later_task.nearest_heading.as_deref(), Some("Later"));

    let under_later = task::tasks(
        &document,
        &task::TaskQuery {
            under: Some(SectionTarget::heading("Later", None, HeadingMatchMode::Exact).unwrap()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(
        under_later
            .iter()
            .map(|task| task.summary_text.as_str())
            .collect::<Vec<_>>(),
        vec!["later task"]
    );
}

#[test]
fn setext_crlf_nested_tasks_and_empty_preamble_keep_exact_projection() {
    let setext = Document::parse(include_str!("fixtures/setext.md")).unwrap();
    let setext_tree = setext.index().render_tree();
    assert_eq!(setext_tree.matches("heading-marker level=").count(), 2);
    assert_eq!(setext_tree.matches("source=setext").count(), 2);
    assert!(setext_tree.contains("section level=2 text=\"Subtitle\""));
    assert!(setext_tree.contains("heading-marker level=1 source=setext lines=2-2 bytes=6..11"));
    assert!(setext_tree.contains("heading-marker level=2 source=setext lines=5-5 bytes=22..30"));

    let crlf = Document::parse(include_str!("fixtures/crlf.md")).unwrap();
    for block in crlf.blocks() {
        crlf.slice(&block.span).unwrap();
    }
    let crlf_tree = crlf.index().render_tree();
    assert!(crlf_tree.contains("kind=Paragraph"));
    assert!(crlf_tree.contains("heading-marker level=1 source=atx lines=1-1 bytes=0..1"));

    let nested = Document::parse(include_str!("fixtures/nested_tasks.md")).unwrap();
    let records = task::tasks(&nested, &Default::default()).unwrap();
    assert_eq!(
        nested.index().node_count(IndexNodeKind::TaskItem),
        records.len()
    );
    assert!(nested
        .index()
        .render_tree()
        .contains("summary=\"Grandchild task\""));

    let frontmatter =
        Document::parse_for_frontmatter("---\ntitle: Only\n---\n\n# First\n").unwrap();
    assert_eq!(frontmatter.index().node_count(IndexNodeKind::Preamble), 1);
    assert_eq!(frontmatter.index().node_count(IndexNodeKind::BodyBlock), 0);
}

#[test]
fn indexed_task_spans_stay_inside_their_block_and_cover_the_checkbox_symbol() {
    let document = Document::parse(include_str!("fixtures/nested_tasks.md")).unwrap();
    for block in document.blocks() {
        for task in &block.task_items {
            assert!(block.span.byte_start <= task.span.byte_start);
            assert!(task.span.byte_end <= block.span.byte_end);
            assert!(task.span.byte_start <= task.symbol_byte_offset);
            assert!(task.symbol_byte_offset < task.span.byte_end);
            assert!(!document.slice(&task.span).unwrap().is_empty());
        }
    }
}

#[test]
fn representative_index_trees_are_reviewable() {
    for (name, source) in [
        ("nested", include_str!("fixtures/nested_tasks.md")),
        ("footnote", include_str!("fixtures/footnote_midbody.md")),
        ("setext", include_str!("fixtures/setext.md")),
        ("crlf", include_str!("fixtures/crlf.md")),
    ] {
        let document = Document::parse_for_frontmatter(source).unwrap();
        let tree = document.index().render_tree();
        assert!(tree.starts_with("document lines="));
        assert!(!tree.contains("body-block ordinal=0 parser-index=0 kind=Heading"));
        println!("=== {name} ===\n{tree}");
    }
}
