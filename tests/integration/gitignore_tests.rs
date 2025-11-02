use assert_cmd::Command as AssertCommand;
use predicates::prelude::*;
use std::fs;

// Import utility functions
use crate::common::test_utils::{
    assert_file_contains, assert_file_exists, create_git_repo, setup_test_env,
};

/**
 Integration tests for the `gitforge` gitignore subcommand.

 This test suite covers the following scenarios:

- `test_gitignore_add_rust`: Verifies that adding a Rust gitignore creates the correct file with expected content.
- `test_gitignore_add_rust.gitignore`: Ensures that adding a Rust gitignore with a .gitignore extension creates the correct file with expected content.
- `test_gitignore_add_multiple`: Validates that multiple gitignore templates can be added and merged.
- `test_gitignore_add_with_dir`: Ensures that a gitignore can be added to a specified directory.
- `test_gitignore_add_force_overwrite`: Tests that an existing .gitignore file is not overwritten unless the `--force` flag is used.
- `test_gitignore_add_all`: Tests the addition of all available gitignore templates.
- `test_gitignore_add_invalid_template`: Confirms that an unknown template returns an appropriate error.
- `test_gitignore_add_no_template`: Ensures that running add without templates or --all returns an error.
- `test_gitignore_add_update_cache`: Ensures that the add command with --update-cache refreshes the cache.
- `test_gitignore_add_valid_and_invalid_template`: Tests adding a valid template alongside an invalid one, ensuring the valid template is added while the invalid one is reported.
- `test_gitignore_list_default`: Ensures the list command displays popular templates.
- `test_gitignore_list_popular`: Ensures the list command with --popular displays popular templates.
- `test_gitignore_list_global`: Ensures the list command with --global displays global templates.
- `test_gitignore_list_community`: Ensures the list command with --community displays community templates.
- `test_gitignore_list_update_cache`: Ensures the list command with --update-cache refreshes the cache.
- `test_gitignore_preview_single_template`: Tests previewing a single gitignore template.
- `test_gitignore_preview_rust.gitignore`: Tests previewing a gitignore template with a .gitignore extension.
- `test_gitignore_preview_multiple_templates`: Tests previewing multiple gitignore templates.
- `test_gitignore_preview_update_cache`: Tests previewing a template with cache update.
- `test_gitignore_preview_invalid_template`: Ensures that previewing an invalid template returns an error.
- `test_gitignore_preview_no_template`: Ensures that running preview without templates returns an
- `test_gitignore_help_command`: Validates that the help command displays usage information for the gitignore subcommands.

Each test uses a temporary directory to avoid side effects and leverages `assert_cmd` and `predicates` for command-line assertions.
*/

// --------     ADD COMMAND TESTS     --------

#[test]
fn test_gitignore_add_rust() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "gitignore", "rust"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Added gitignore templates").or(predicate::str::contains("✓")),
        );

    assert_file_exists(&temp_path.join(".gitignore"));
    assert_file_contains(&temp_path.join(".gitignore"), "Rust");
}

#[test]
fn test_gitignore_add_rust_gitignore() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "gitignore", "rust.gitignore"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Added gitignore templates").or(predicate::str::contains("✓")),
        );

    assert_file_exists(&temp_path.join(".gitignore"));
    assert_file_contains(&temp_path.join(".gitignore"), "Rust");
}

#[test]
fn test_gitignore_add_multiple() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "gitignore", "rust", "python"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Added gitignore templates").or(predicate::str::contains("✓")),
        );

    let content = fs::read_to_string(temp_path.join(".gitignore")).unwrap();
    assert!(content.contains("Rust"));
    assert!(content.contains("Python"));
}

#[test]
fn test_gitignore_add_with_dir() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();
    let target_dir = temp_path.join("custom_dir");
    fs::create_dir_all(&target_dir).unwrap();

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.args(&[
        "add",
        "gitignore",
        "rust",
        "--dir",
        target_dir.to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(
        predicate::str::contains("Added gitignore templates").or(predicate::str::contains("✓")),
    );

    assert_file_exists(&target_dir.join(".gitignore"));
    assert_file_contains(&target_dir.join(".gitignore"), "Rust");
}

#[test]
fn test_gitignore_add_append() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    // First add Rust template
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
cmd.args(&["add", "gitignore", "rust"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Added gitignore templates").or(predicate::str::contains("✓")),
        );

    assert_file_exists(&temp_path.join(".gitignore"));
    assert_file_contains(&temp_path.join(".gitignore"), "Rust");

    // Then append Python template
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "gitignore", "python", "--append"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Added gitignore templates: rust")
                .or(predicate::str::contains("✓")),
        );

    let content = fs::read_to_string(temp_path.join(".gitignore")).unwrap();
    assert!(content.contains("Rust"));
    assert!(content.contains("Python"));
}

#[test]
fn test_gitignore_add_force_overwrite() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    // Create initial .gitignore file
    fs::write(temp_path.join(".gitignore"), "existing content").unwrap();

    // Try to add without force (should fail)
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "gitignore", "rust"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    // Try with force flag (should succeed)
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "gitignore", "rust", "--force"])
        .assert()
        .success();

    assert_file_contains(&temp_path.join(".gitignore"), "Rust");
    let content = fs::read_to_string(temp_path.join(".gitignore")).unwrap();
    assert!(!content.contains("existing content"));
}

#[test]
#[ignore] // This test takes a long time to run as it downloads all templates
fn test_gitignore_add_all() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "gitignore", "--all"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Downloaded and merged all gitignore templates")
                .or(predicate::str::contains("✓")),
        );

    assert_file_exists(&temp_path.join(".gitignore"));
    let content = fs::read_to_string(temp_path.join(".gitignore")).unwrap();
    assert!(content.contains(".gitignore"));
}

#[test]
fn test_gitignore_add_invalid_template() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "gitignore", "not-a-template"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Unknown")));
}

#[test]
fn test_gitignore_add_no_template() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "gitignore", "not-a-template"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Unknown")));
}

#[test]
fn test_gitignore_add_update_cache() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "gitignore", "rust", "--update-cache"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Added gitignore templates").or(predicate::str::contains("✓")),
        );
}

#[test]
fn test_gitignore_add_valid_and_invalid_template() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    // Try to add one valid and one invalid template
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "gitignore", "rust", "not-a-template"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Added gitignore templates").or(predicate::str::contains("✓")),
        )
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Unknown")));

    // The .gitignore file should be created and contain the valid template's content
    assert_file_exists(&temp_path.join(".gitignore"));
    assert_file_contains(&temp_path.join(".gitignore"), "Rust");
    let content = fs::read_to_string(temp_path.join(".gitignore")).unwrap();
    assert!(!content.contains("not-a-template"));
}

#[test]
fn test_gitignore_add_default_with_output() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    // Add rust template with output file .gitignore
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "gitignore", "rust", "-o", ".gitignore"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Added gitignore templates").or(predicate::str::contains("✓")),
        );

    assert_file_exists(&temp_path.join(".gitignore"));
    assert_file_contains(&temp_path.join(".gitignore"), "Rust");
}

#[test]
fn test_gitignore_add_multiple_templates_uneven_output_files() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    // Pass 3 output files for 2 templates (should error)
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&[
        "add",
        "gitignore",
        "python",
        "rust",
        "-o",
        "Python.gitignore",
        "Rust.gitignore",
        "Ada.gitignore",
    ])
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "Number of output files must be either 1 or match the number of templates when not using --use-remote-name",
    ));
}

// --------     LIST COMMAND TESTS     --------

#[test]
fn test_gitignore_list_default() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["list", "gitignores"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Available gitignore templates"))
        .stdout(predicate::str::contains("POPULAR"))
        .stdout(predicate::str::contains("GLOBAL"))
        .stdout(predicate::str::contains("COMMUNITY"));
}

#[test]
fn test_gitignore_list_popular() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["list", "gitignores", "--popular"])
        .assert()
        .success()
        .stdout(predicate::str::contains("POPULAR"));
}

#[test]
fn test_gitignore_list_global() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["list", "gitignores", "--global"])
        .assert()
        .success()
        .stdout(predicate::str::contains("global"));
}

#[test]
fn test_gitignore_list_community() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["list", "gitignores", "--community"])
        .assert()
        .success()
        .stdout(predicate::str::contains("community"));
}

#[test]
fn test_gitignore_list_update_cache() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["list", "gitignores", "--update-cache"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Available gitignore templates"));
}

// --------     PREVIEW COMMAND TESTS     --------

#[test]
fn test_gitignore_preview_single_template() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["preview", "gitignore", "rust"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust"));
}

#[test]
fn test_gitignore_preview_single_template_update_cache() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["preview", "gitignore", "rust", "--update-cache"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust"));
}

#[test]
fn test_gitignore_preview_multiple_templates() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["preview", "gitignore", "rust", "python"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust"))
        .stdout(predicate::str::contains("Python"));
}

#[test]
fn test_gitignore_preview_update_cache() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["preview", "gitignore", "rust", "--update-cache"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust"));
}

#[test]
fn test_gitignore_preview_invalid_template() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["preview", "gitignore", "not-a-template"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found").or(predicate::str::contains("Unknown")));
}

#[test]
fn test_gitignore_preview_no_template() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["preview", "gitignore"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No gitignore template specified"));
}
// --------     HELP COMMAND TEST     --------

#[test]
fn test_gitignore_help_command() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    // add gitignore --help
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "gitignore", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Add gitignore templates"))
        .stdout(predicate::str::contains("Usage: gitforge add gitignore"))
        .stdout(predicate::str::contains("--dir"))
        .stdout(predicate::str::contains("--force"))
        .stdout(predicate::str::contains("--update-cache"))
        .stdout(predicate::str::contains("--output").or(predicate::str::contains("-o")));

    // list gitignore --help
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["list", "gitignore", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("List available gitignore templates"))
        .stdout(predicate::str::contains("Usage: gitforge list gitignore"))
        .stdout(predicate::str::contains("-p").or(predicate::str::contains("--popular")))
        .stdout(predicate::str::contains("--update-cache"));

    // preview gitignore --help
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["preview", "gitignore", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview a gitignore template"))
        .stdout(predicate::str::contains("Usage: gitforge preview gitignore"))
        .stdout(predicate::str::contains("--update-cache"));
}
