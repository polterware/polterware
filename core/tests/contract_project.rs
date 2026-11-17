use polterware_core::local_v1::Project;

#[test]
fn test_project_has_a_name() {
    let project = Project {
        id: "project".to_string(),
        name: "Project".to_string(),
        ..Project::default()
    };
    assert_eq!(project.name, "Project");
}
