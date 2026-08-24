use mdtools::block;
use mdtools::block_edit;
use mdtools::core_error::CoreError;
use mdtools::document::Document;
use mdtools::frontmatter::{self, FrontmatterAction, FrontmatterEdit, FrontmatterPath};
use mdtools::link;
use mdtools::model::{
    BlockMoveMode, HeadingMatchMode, InsertLocation, InsertMode, MutationDisposition,
};
use mdtools::search::{self, SearchQuery};
use mdtools::section::{SectionIndex, SectionTarget};
use mdtools::section_edit;
use mdtools::stats;
use mdtools::table::{self, ColumnSelector, TableQuery};

#[test]
fn inspection_surface_projects_one_document_without_cli_context() {
    let source = "---\ntitle: Demo\n---\n\n# Tasks\n\nRead [guide](guide.md).\n\n| Name | State |\n| --- | --- |\n| A | open |\n";
    let document = Document::parse_for_frontmatter(source).unwrap();

    assert_eq!(block::blocks(&document).len(), 3);
    assert_eq!(
        link::links(&document)[0].destination.as_deref(),
        Some("guide.md")
    );
    assert_eq!(stats::document_stats(&document).heading_count, 1);
    assert_eq!(frontmatter::read(&document).unwrap().data["title"], "Demo");
    assert_eq!(
        search::search(&document, &SearchQuery::literal("guide")).len(),
        2
    );

    let table_index = table::tables(&document).unwrap()[0].block_index;
    let table = table::table(
        &document,
        table_index,
        &TableQuery {
            columns: vec![ColumnSelector::Name("State".into())],
            predicates: Vec::new(),
        },
    )
    .unwrap();
    assert_eq!(table.rows, vec![vec!["open"]]);
}

#[test]
fn prepared_payload_edits_return_candidates_without_io() {
    let source = "# One\n\nalpha\n\n# Two\n\nbeta\n";
    let document = Document::parse(source).unwrap();
    let replacement = block_edit::prepare_replace(&document, 1, None)
        .unwrap()
        .apply("changed\n");
    assert_eq!(replacement.disposition, MutationDisposition::Replaced);
    assert!(replacement.content.contains("changed"));
    assert_eq!(document.source(), source);

    let insertion = block_edit::prepare_insert(&document, InsertLocation::After(1), None)
        .unwrap()
        .apply("inserted")
        .unwrap();
    assert!(insertion.content.contains("inserted"));
    assert_eq!(document.source(), source);
}

#[test]
fn frontmatter_table_and_section_edits_are_in_process() {
    let source = "---\ntitle: Old\n---\n\n# One\n\n| A |\n| --- |\n| old |\n\n# Two\n\nend\n";
    let document = Document::parse_for_frontmatter_mutation(source).unwrap();
    let frontmatter = frontmatter::edit(
        &document,
        &FrontmatterEdit {
            key_path: FrontmatterPath::new("title").unwrap(),
            action: FrontmatterAction::Set(serde_json::json!("New")),
            expect_etag: None,
        },
    )
    .unwrap();
    assert!(frontmatter.content.contains("title: New"));

    let table_index = table::tables(&document).unwrap()[0].block_index;
    let row = table::prepare_replace_row(&document, table_index, 0, None)
        .unwrap()
        .replace("| new |")
        .unwrap();
    assert!(row.content.contains("| new |"));

    let index = SectionIndex::new(&document);
    let one = index
        .resolve(&SectionTarget::heading("One", None, HeadingMatchMode::Exact).unwrap())
        .unwrap();
    let two = index
        .resolve(&SectionTarget::heading("Two", None, HeadingMatchMode::Exact).unwrap())
        .unwrap();
    let moved = section_edit::move_section(
        &document,
        one,
        two,
        InsertMode::AfterSibling,
        true,
        None,
        None,
    )
    .unwrap();
    assert!(moved.content.find("# Two").unwrap() < moved.content.find("# One").unwrap());
    assert_eq!(document.source(), source);
}

#[test]
fn block_relocation_preserves_the_block_multiset() {
    let document = Document::parse("# A\n\none\n\n# B\n\ntwo\n").unwrap();
    let moved = block_edit::move_block(&document, 0, 3, BlockMoveMode::After, None, None).unwrap();
    assert_eq!(Document::parse(moved.content).unwrap().blocks().len(), 4);
}

#[test]
fn frontmatter_paths_reject_empty_segments() {
    for invalid in ["", ".a", "a.", "a..b"] {
        assert!(matches!(
            FrontmatterPath::new(invalid),
            Err(CoreError::InvalidKeyPath { .. })
        ));
    }
}

#[test]
fn frontmatter_edit_revalidates_lenient_documents() {
    let request = FrontmatterEdit {
        key_path: FrontmatterPath::new("title").unwrap(),
        action: FrontmatterAction::Set(serde_json::json!("new")),
        expect_etag: None,
    };
    for source in ["---\ntitle: [\n---\nbody\n", "---\ntitle: old\nbody\n"] {
        let document = Document::parse(source).unwrap();
        assert!(matches!(
            frontmatter::edit(&document, &request),
            Err(CoreError::FrontmatterParseFailed(_))
        ));
    }
}
