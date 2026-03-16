//! Persona-driven tests: real documents from real users exercising real assumptions.

use std::io::Write;
use std::process::{Command, Stdio};

fn md() -> Command {
    Command::new(env!("CARGO_BIN_EXE_md"))
}

fn md_json(args: &[&str]) -> serde_json::Value {
    let mut a = args.to_vec();
    if !a.contains(&"--json") {
        a.push("--json");
    }
    let o = md().args(&a).output().unwrap();
    assert!(o.status.success(), "{:?} failed: {}", args, String::from_utf8_lossy(&o.stderr));
    serde_json::from_slice(&o.stdout).unwrap()
}

fn md_text(args: &[&str]) -> String {
    let o = md().args(args).output().unwrap();
    assert!(o.status.success(), "{:?} failed: {}", args, String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).to_string()
}

fn md_stdin(args: &[&str], input: &str) -> std::process::Output {
    let mut child = md()
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(input.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

fn tmpfile(content: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static CTR: AtomicU64 = AtomicU64::new(0);
    let id = CTR.fetch_add(1, Ordering::SeqCst);
    let p = format!("/tmp/mdtools_persona_{}_{}.md", std::process::id(), id);
    std::fs::write(&p, content).unwrap();
    p
}

// ============================================================
// OBSIDIAN: wikilinks, callouts, block references, task lists
// ============================================================

#[test]
fn obsidian_wikilinks_are_not_links() {
    // Wikilinks [[note]] are NOT detected as links in Phase 1
    // They should be plain text inside paragraphs
    let json = md_json(&["links", "tests/fixtures/persona_obsidian.md"]);
    let links = json["links"].as_array().unwrap();
    // Zero links — wikilinks aren't standard markdown links
    assert_eq!(links.len(), 0, "wikilinks should not appear as links in Phase 1");
}

#[test]
fn obsidian_callouts_are_blockquotes() {
    let json = md_json(&["blocks", "tests/fixtures/persona_obsidian.md"]);
    let blocks = json["blocks"].as_array().unwrap();
    let blockquotes: Vec<_> = blocks.iter().filter(|b| b["kind"] == "BlockQuote").collect();
    // > [!WARNING] and > [!NOTE] are both blockquotes
    assert_eq!(blockquotes.len(), 2, "callouts should parse as BlockQuote");
}

#[test]
fn obsidian_callouts_not_in_outline() {
    // Headings inside blockquotes must NOT create outline entries
    let json = md_json(&["outline", "tests/fixtures/persona_obsidian.md"]);
    let headings: Vec<&str> = json["entries"]
        .as_array().unwrap().iter()
        .map(|e| e["heading"]["text"].as_str().unwrap())
        .collect();
    // No callout titles should appear as headings
    assert!(!headings.contains(&"Breaking Change"));
    assert!(!headings.contains(&"[!WARNING] Breaking Change"));
}

#[test]
fn obsidian_block_ref_preserved_in_content() {
    // ^key-insight at end of paragraph should be part of block content
    let source = std::fs::read_to_string("tests/fixtures/persona_obsidian.md").unwrap();
    let json = md_json(&["block", "1", "tests/fixtures/persona_obsidian.md"]);
    let content = json["content"].as_str().unwrap();
    assert!(content.contains("^key-insight"), "block ref should be in paragraph content");
}

#[test]
fn obsidian_block_ref_survives_roundtrip() {
    // Replacing a block that contains ^ref should preserve the ref
    let source = std::fs::read_to_string("tests/fixtures/persona_obsidian.md").unwrap();
    let tmp = tmpfile(&source);

    let block = md_json(&["block", "1", &tmp]);
    let content = block["content"].as_str().unwrap();
    assert!(content.contains("^key-insight"));

    // Replace with same content
    let output = md_stdin(&["replace-block", "1", &tmp, "-i"], content);
    assert!(output.status.success());

    let after = std::fs::read_to_string(&tmp).unwrap();
    assert!(after.contains("^key-insight"), "block ref must survive roundtrip replacement");
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn obsidian_task_list_word_count() {
    let json = md_json(&["stats", "tests/fixtures/persona_obsidian.md"]);
    let wc = json["stats"]["word_count"].as_u64().unwrap();
    // Task list items should count words, not checkbox markers
    // The list contains items like "Implement parser boundary", "Add source span projection", etc.
    assert!(wc > 50, "word count {} should include task list item text", wc);
}

#[test]
fn obsidian_embed_syntax_not_link() {
    // ![[attachment.png]] should NOT be a link (it's not valid markdown link syntax)
    let json = md_json(&["links", "tests/fixtures/persona_obsidian.md"]);
    let links = json["links"].as_array().unwrap();
    for link in links {
        let dest = link["destination"].as_str().unwrap_or("");
        assert!(!dest.contains("attachment.png"), "embed syntax should not create links");
    }
}

// ============================================================
// README: badge images, nested lists, task lists, relative links
// ============================================================

#[test]
fn readme_badge_links_detected() {
    // [![badge](img)](url) — the OUTER link should be detected
    let json = md_json(&["links", "tests/fixtures/persona_readme.md"]);
    let links = json["links"].as_array().unwrap();
    let dests: Vec<&str> = links.iter()
        .filter_map(|l| l["destination"].as_str())
        .collect();
    assert!(dests.iter().any(|d| d.contains("github.com")), "should find badge link to GitHub");
    assert!(dests.iter().any(|d| d.contains("codecov.io")), "should find badge link to Codecov");
}

#[test]
fn readme_badge_image_urls_not_in_links() {
    // The inner image URLs (img.shields.io) should NOT appear as link destinations
    let json = md_json(&["links", "tests/fixtures/persona_readme.md"]);
    let links = json["links"].as_array().unwrap();
    for link in links {
        let dest = link["destination"].as_str().unwrap_or("");
        assert!(!dest.contains("img.shields.io"), "image URLs should not be in links output");
    }
}

#[test]
fn readme_relative_links_detected() {
    let json = md_json(&["links", "tests/fixtures/persona_readme.md"]);
    let links = json["links"].as_array().unwrap();
    let dests: Vec<&str> = links.iter()
        .filter_map(|l| l["destination"].as_str())
        .collect();
    assert!(dests.contains(&"./CONTRIBUTING.md"), "relative link should be detected");
    assert!(dests.contains(&"../docs/API.md"), "parent-relative link should be detected");
}

#[test]
fn readme_deeply_nested_list_word_count() {
    // Nested list items at 4+ levels should still count words
    let json = md_json(&["stats", "tests/fixtures/persona_readme.md"]);
    let wc = json["stats"]["word_count"].as_u64().unwrap();
    // The document has substantive content across nested lists
    assert!(wc > 50, "word count {} should cover nested list content", wc);
}

#[test]
fn readme_task_list_is_list_block() {
    let json = md_json(&["blocks", "tests/fixtures/persona_readme.md"]);
    let blocks = json["blocks"].as_array().unwrap();
    let lists: Vec<_> = blocks.iter().filter(|b| b["kind"] == "List").collect();
    assert!(lists.len() >= 2, "task lists and nested lists should be List blocks");
}

#[test]
fn readme_table_links_in_cells() {
    // The table has backtick code in cells but no actual links
    // The paragraph after has links — verify they're attributed to the right block
    let json = md_json(&["links", "tests/fixtures/persona_readme.md"]);
    let links = json["links"].as_array().unwrap();
    for link in links {
        let bi = link["source_block_index"].as_u64().unwrap();
        let block = md_json(&["block", &bi.to_string(), "tests/fixtures/persona_readme.md"]);
        // Links should come from paragraphs, not from the table
        assert_ne!(
            block["block"]["kind"].as_str().unwrap(), "Table",
            "links in table cells detected as Table block links — might be incorrect"
        );
    }
}

// ============================================================
// RESEARCHER: footnotes, citations, long structure, blockquote attribution
// ============================================================

#[test]
fn researcher_footnote_definitions_are_blocks() {
    let json = md_json(&["blocks", "tests/fixtures/persona_researcher.md"]);
    let blocks = json["blocks"].as_array().unwrap();
    let fns: Vec<_> = blocks.iter().filter(|b| b["kind"] == "FootnoteDefinition").collect();
    assert_eq!(fns.len(), 4, "should have 4 footnote definition blocks");
}

#[test]
fn researcher_footnote_refs_in_paragraphs() {
    // [^1] in paragraph text should be searchable
    let json = md_json(&["search", "[^1]", "tests/fixtures/persona_researcher.md"]);
    let matches = json["matches"].as_array().unwrap();
    assert!(!matches.is_empty(), "should find footnote reference [^1] in text");
}

#[test]
fn researcher_citation_not_a_link() {
    // [@smith2020] is Pandoc citation syntax, NOT a markdown link
    let json = md_json(&["links", "tests/fixtures/persona_researcher.md"]);
    let links = json["links"].as_array().unwrap();
    for link in links {
        let dest = link["destination"].as_str().unwrap_or("");
        assert!(!dest.contains("smith2020"), "citation syntax should not create links");
    }
}

#[test]
fn researcher_blockquote_attribution_not_heading() {
    // The blockquote contains "— Johnson et al." which should NOT be an outline heading
    let json = md_json(&["outline", "tests/fixtures/persona_researcher.md"]);
    let headings: Vec<&str> = json["entries"]
        .as_array().unwrap().iter()
        .map(|e| e["heading"]["text"].as_str().unwrap())
        .collect();
    for h in &headings {
        assert!(!h.contains("Johnson"), "blockquote attribution should not be a heading");
    }
}

#[test]
fn researcher_deep_section_hierarchy() {
    let json = md_json(&["outline", "tests/fixtures/persona_researcher.md"]);
    let levels: Vec<u8> = json["entries"]
        .as_array().unwrap().iter()
        .map(|e| e["heading"]["level"].as_u64().unwrap() as u8)
        .collect();
    assert!(levels.contains(&3), "should have H3 subsections (Participants, Procedure, Analysis)");

    // Methods section should contain its subsections
    let methods = md_text(&["section", "Methods", "tests/fixtures/persona_researcher.md"]);
    assert!(methods.contains("### Participants"));
    assert!(methods.contains("### Procedure"));
    assert!(methods.contains("### Analysis"));
    assert!(!methods.contains("## Results"), "Methods section should not include Results");
}

#[test]
fn researcher_footnote_orphaning_detection() {
    // Agent workflow: delete a paragraph that references [^2], verify [^2]: definition still exists
    let source = std::fs::read_to_string("tests/fixtures/persona_researcher.md").unwrap();
    let tmp = tmpfile(&source);

    // Find the paragraph containing [^2]
    let search = md_json(&["search", "[^2]", &tmp, "--kind", "paragraph"]);
    let matches = search["matches"].as_array().unwrap();
    assert!(!matches.is_empty());
    let block_idx = matches[0]["block_index"].as_u64().unwrap();

    // Delete that paragraph
    let output = md_stdin(&["replace-block", &block_idx.to_string(), &tmp, "-i"], "");
    assert!(output.status.success());

    // The footnote DEFINITION [^2]: should still exist (orphaned but present)
    let after = std::fs::read_to_string(&tmp).unwrap();
    assert!(after.contains("[^2]:"), "footnote definition should survive even when reference is deleted");
    // The tool doesn't warn about orphaned footnotes — that's a semantic gap, not a bug
    std::fs::remove_file(&tmp).ok();
}

// ============================================================
// BLOGGER: shortcodes survive mutations, TOML/YAML frontmatter
// ============================================================

#[test]
fn blogger_shortcodes_survive_mutation() {
    // {{< callout >}} and {{< figure >}} should survive replace-section
    let source = std::fs::read_to_string("tests/fixtures/persona_blogger.md").unwrap();
    let tmp = tmpfile(&source);

    // Replace the Conclusion section
    let output = md_stdin(
        &["replace-section", "Conclusion", &tmp, "-i"],
        "## Conclusion\n\nUpdated conclusion.\n",
    );
    assert!(output.status.success());

    // Shortcodes in OTHER sections must be untouched
    let after = std::fs::read_to_string(&tmp).unwrap();
    assert!(after.contains("{{< callout"), "callout shortcode must survive");
    assert!(after.contains("{{< figure"), "figure shortcode must survive");
    std::fs::remove_file(&tmp).ok();
}

#[test]
fn blogger_frontmatter_complex_yaml() {
    let json = md_json(&["frontmatter", "tests/fixtures/persona_blogger.md"]);
    assert_eq!(json["present"], true);
    let data = &json["frontmatter"]["data"];
    assert_eq!(data["title"], "Why I Switched to Rust for CLI Tools");
    assert_eq!(data["draft"], false);
    assert!(data["tags"].as_array().unwrap().len() == 3);
    // Nested object in frontmatter (cover.image)
    assert_eq!(data["cover"]["image"], "/images/rust-cli.png");
}

#[test]
fn blogger_shortcodes_are_paragraph_blocks() {
    // {{< callout >}} is not valid markdown — should be parsed as part of a paragraph
    let json = md_json(&["blocks", "tests/fixtures/persona_blogger.md"]);
    let blocks = json["blocks"].as_array().unwrap();
    // Find blocks containing shortcode syntax
    let shortcode_blocks: Vec<_> = blocks.iter().filter(|b| {
        let preview = b["preview"].as_str().unwrap_or("");
        preview.contains("{{<") || preview.contains("{{%")
    }).collect();
    assert!(!shortcode_blocks.is_empty(), "shortcodes should appear in block previews");
    for b in &shortcode_blocks {
        assert_eq!(b["kind"], "Paragraph", "shortcodes should be Paragraph blocks");
    }
}

// ============================================================
// JOURNALIST: few headings, long paragraphs, thematic breaks
// ============================================================

#[test]
fn journalist_mostly_paragraphs() {
    let json = md_json(&["blocks", "tests/fixtures/persona_journalist.md"]);
    let blocks = json["blocks"].as_array().unwrap();
    let para_count = blocks.iter().filter(|b| b["kind"] == "Paragraph").count();
    let total = blocks.len();
    // Most blocks should be paragraphs
    assert!(
        para_count as f64 / total as f64 > 0.7,
        "journalist doc should be mostly paragraphs ({}/{})",
        para_count, total
    );
}

#[test]
fn journalist_single_section() {
    // Only one heading → one section
    let json = md_json(&["stats", "tests/fixtures/persona_journalist.md"]);
    assert_eq!(json["stats"]["section_count"], 1);
    assert_eq!(json["stats"]["heading_count"], 1);
}

#[test]
fn journalist_thematic_break_is_not_heading() {
    let json = md_json(&["outline", "tests/fixtures/persona_journalist.md"]);
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 1, "thematic break --- should not create additional outline entries");
}

#[test]
fn journalist_long_paragraph_search() {
    // Search should find text in very long paragraphs
    let json = md_json(&["search", "steering wheel", "tests/fixtures/persona_journalist.md"]);
    let matches = json["matches"].as_array().unwrap();
    assert_eq!(matches.len(), 1, "should find 'steering wheel' in long paragraph");
}

#[test]
fn journalist_unicode_punctuation() {
    // Smart quotes, em-dashes should be searchable and preserve through mutations
    let source = std::fs::read_to_string("tests/fixtures/persona_journalist.md").unwrap();
    assert!(source.contains("\u{2014}") || source.contains("—"), "fixture should contain em-dash");

    let json = md_json(&["search", "—", "tests/fixtures/persona_journalist.md"]);
    let matches = json["matches"].as_array().unwrap();
    assert!(!matches.is_empty(), "should find em-dash in text");
}

// ============================================================
// EDGE: empty sections, no headings, adjacent code, long lines
// ============================================================

#[test]
fn empty_section_contains_only_heading() {
    let json = md_json(&["section", "Empty Section", "tests/fixtures/edge_empty_sections.md"]);
    let indices = json["section"]["block_indices"].as_array().unwrap();
    assert_eq!(indices.len(), 1, "empty section should contain only its heading block");
}

#[test]
fn empty_section_followed_by_content_section() {
    // "Section With Content" should have heading + paragraph
    let json = md_json(&["section", "Section With Content", "tests/fixtures/edge_empty_sections.md"]);
    let indices = json["section"]["block_indices"].as_array().unwrap();
    assert_eq!(indices.len(), 2, "content section should have heading + paragraph");
    let content = json["content"].as_str().unwrap();
    assert!(content.contains("Actual content here."));
}

#[test]
fn no_headings_everything_is_preamble() {
    let json = md_json(&["section", ":preamble", "tests/fixtures/edge_no_headings.md"]);
    let indices = json["section"]["block_indices"].as_array().unwrap();
    let content = json["content"].as_str().unwrap();
    assert!(indices.len() >= 3, "preamble should contain all blocks");
    assert!(content.contains("no headings at all"));
    assert!(content.contains("Item one"));
}

#[test]
fn no_headings_outline_empty() {
    let json = md_json(&["outline", "tests/fixtures/edge_no_headings.md"]);
    let entries = json["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 0, "no-heading doc should have empty outline");
}

#[test]
fn no_headings_section_count_is_one() {
    // Non-empty preamble counts as one section
    let json = md_json(&["stats", "tests/fixtures/edge_no_headings.md"]);
    assert_eq!(json["stats"]["section_count"], 1);
}

#[test]
fn no_headings_heading_selector_returns_not_found() {
    let output = md()
        .args(["section", "Anything", "tests/fixtures/edge_no_headings.md"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn adjacent_code_blocks_have_sequential_indices() {
    let json = md_json(&["blocks", "tests/fixtures/edge_adjacent_code.md"]);
    let code_blocks: Vec<u64> = json["blocks"].as_array().unwrap()
        .iter()
        .filter(|b| b["kind"] == "CodeFence")
        .map(|b| b["index"].as_u64().unwrap())
        .collect();
    assert_eq!(code_blocks.len(), 3);
    // Indices should be sequential
    assert_eq!(code_blocks[1], code_blocks[0] + 1);
    assert_eq!(code_blocks[2], code_blocks[1] + 1);
}

#[test]
fn adjacent_code_blocks_search_inside() {
    // Search for language-specific content across adjacent code blocks
    let json = md_json(&["search", "println", "tests/fixtures/edge_adjacent_code.md", "--kind", "code-fence"]);
    let matches = json["matches"].as_array().unwrap();
    // Should find in both Python (print) and Rust (println)
    assert!(matches.len() >= 1, "should find println in code blocks");
}

#[test]
fn long_line_preview_truncated() {
    let json = md_json(&["blocks", "tests/fixtures/edge_long_line.md"]);
    let blocks = json["blocks"].as_array().unwrap();
    let long_para = blocks.iter().find(|b| b["kind"] == "Paragraph").unwrap();
    let preview = long_para["preview"].as_str().unwrap();
    // Preview should be truncated to 80 chars + "..."
    assert!(preview.len() <= 83, "preview should be truncated: len={}", preview.len());
    assert!(preview.ends_with("..."), "truncated preview should end with ...");
}

#[test]
fn long_line_content_intact() {
    // Full block content should NOT be truncated
    let json = md_json(&["block", "1", "tests/fixtures/edge_long_line.md"]);
    let content = json["content"].as_str().unwrap();
    assert!(content.len() > 200, "full content should not be truncated");
    assert!(content.contains("end."), "full content should reach the end");
}
