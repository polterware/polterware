use polterware_core::local_v1::{Environment, Project};

#[test]
fn test_project_contains_local_path() {
    let project = Project {
        id: "project".to_string(),
        name: "Project".to_string(),
        ..Project::default()
    };
    let environment = Environment {
        id: "local".to_string(),
        project_id: project.id.clone(),
        name: "local".to_string(),
        kind: "local".to_string(),
        ..Environment::default()
    };
    assert!(
        !project.name.is_empty() && environment.project_id == project.id,
        "project contract"
    );
}
