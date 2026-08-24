use std::str::FromStr;

use mdtools::document::Document;
use mdtools::fingerprint::TargetEtag;
use mdtools::model::TaskStatus;
use mdtools::task::{self, SetTaskEdit, TaskLoc, TaskQuery};

#[test]
fn task_loc_roundtrips() {
    let loc = TaskLoc::from_str("14.4.0").unwrap();
    assert_eq!(loc.to_string(), "14.4.0");
    assert!(TaskLoc::from_str("14").is_err());
    assert!(TaskLoc::from_str("14.nope").is_err());
}

#[test]
fn direct_task_edit_returns_candidate_without_touching_source() {
    let source = "# Tasks\n\n- [ ] first\n- [ ] second\n".to_string();
    let document = Document::parse(source.clone()).unwrap();
    let entries = task::tasks(&document, &TaskQuery::default()).unwrap();
    let selected = &entries[0];
    let outcome = task::set_task(
        &document,
        &SetTaskEdit {
            loc: TaskLoc::from_str(&selected.loc).unwrap(),
            status: TaskStatus::Done,
            expect_etag: Some(selected.etag.parse::<TargetEtag>().unwrap()),
        },
    )
    .unwrap();

    assert_eq!(document.source(), source);
    assert_eq!(outcome.base_revision, *document.revision());
    assert!(outcome.changed());
    assert_eq!(outcome.content, "# Tasks\n\n- [x] first\n- [ ] second\n");
}
