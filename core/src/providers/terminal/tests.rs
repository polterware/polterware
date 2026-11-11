use super::*;

#[test]
fn build_ssh_command_returns_an_interactive_session() {
    let (name, args) = build_ssh_command(&SshProfile {
        user: "ubuntu".to_string(),
        host: "10.0.0.12".to_string(),
        port: 2222,
        identity_file: "/tmp/id_ed25519".to_string(),
    })
    .expect("ssh command");
    assert!(name.contains("ssh"), "binary {name:?}");
    let joined = args.join(" ");
    assert!(
        joined.contains("-tt") && joined.contains("-p 2222"),
        "args {args:?}"
    );
    assert!(
        joined.contains("-i /tmp/id_ed25519"),
        "missing identity: {args:?}"
    );
    assert!(
        joined.contains("ubuntu@10.0.0.12"),
        "missing target: {args:?}"
    );
}

#[test]
fn build_ssh_command_requires_user_and_host() {
    assert!(
        build_ssh_command(&SshProfile {
            user: String::new(),
            host: "h".to_string(),
            ..SshProfile::default()
        })
        .is_err(),
        "expected error"
    );
    assert!(
        build_ssh_command(&SshProfile {
            user: "u".to_string(),
            host: String::new(),
            ..SshProfile::default()
        })
        .is_err(),
        "expected error"
    );
}

#[test]
fn build_ssh_command_defaults_the_port_and_rejects_a_bad_one() {
    let (_, args) = build_ssh_command(&SshProfile {
        user: "ubuntu".to_string(),
        host: "example.com".to_string(),
        ..SshProfile::default()
    })
    .expect("ssh command");
    let joined = args.join(" ");
    assert!(joined.contains("-p 22"), "args {args:?}");
    assert!(!joined.contains(" -i "), "no identity was given: {args:?}");
    let err = build_ssh_command(&SshProfile {
        user: "ubuntu".to_string(),
        host: "example.com".to_string(),
        port: 70000,
        ..SshProfile::default()
    })
    .expect_err("port out of range");
    assert_eq!(err.to_string(), "ssh port must be between 1 and 65535");
}

#[test]
fn manager_reports_a_missing_session() {
    let manager = Manager::new();
    let err = manager.get("term-1").expect_err("no session");
    assert_eq!(err.to_string(), "terminal session not found");
    assert!(manager.list("project-1").is_empty());
}
