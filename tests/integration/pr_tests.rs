use assert_cmd::Command as AssertCommand;
use std::fs;

/**
Integration tests for the `gitforge` pr subcommand.

This test suite covers the following scenarios:

- `test_pr_add_default`: Verifies that adding a default PR template creates the correct file with expected content.
- `test_pr_add_with_dir`: Ensures that a PR template can be added to a specified directory.
- `test_pr_add_force_overwrite`: Tests that an existing PR template file is not overwritten unless the `--force` flag is used.
- `test_pr_add_invalid_type`: Confirms that an unknown PR template type returns an appropriate error.
- `test_pr_add_unknown_argument`: Checks that an unknown argument results in an error.
- `test_pr_add_valid_and_invalid_template`: Tests adding both a valid and an invalid PR template in one command, ensuring the valid template is still created.
- `test_pr_add_default_with_output_without_ext`: Tests adding a default PR template with an output name that does not have an extension.
- `test_pr_add_default_with_output_with_ext`: Tests adding a default PR template with an output name that has an extension.
- `test_pr_add_uneven_templates_and_outputs`: Ensures that an error is raised when the number of templates does not match the number of output file names.
- `test_pr_list`: Ensures the list command displays available PR templates.
- `test_pr_preview_single`: Validates that the preview command displays the content of a PR template.
- `test_pr_preview_invalid_id`: Ensures that an invalid PR template ID results in an error.
- `test_pr_help_command`: Validates that the help command displays usage information for the pr subcommands.

Each test uses a temporary directory to avoid side effects and leverages `assert_cmd` and `predicates` for command-line assertions.
*/
use predicates::prelude::*;

// Import utility functions
use crate::common::test_utils::{assert_file_exists, create_git_repo, setup_test_env};

// --------     ADD COMMAND TESTS     --------

#[test]
fn test_pr_add_default() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "pr", "default"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            ".github/pull_request_template.md - has been added.",
        ));

    assert_file_exists(&temp_path.join(".github/pull_request_template.md"));
}

#[test]
fn test_pr_add_with_dir() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();
    let target_dir = temp_path.join("custom_dir");
    fs::create_dir_all(&target_dir).unwrap();

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&[
        "add",
        "pr",
        "default",
        "--dir",
        target_dir.to_str().unwrap(),
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains(format!(
        "pull_request_template.md - has been added.",
    )));

    assert_file_exists(&target_dir.join("./pull_request_template.md"));
}

#[test]
fn test_pr_add_force_overwrite() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    let pr_template_path = temp_path.join(".github/pull_request_template.md");
    fs::create_dir_all(pr_template_path.parent().unwrap()).unwrap();
    fs::write(&pr_template_path, "existing content").unwrap();

    // Try to add without force (should fail)
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.args(&["add", "pr", "default"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
    cmd.current_dir(&temp_path);

    // Try with force flag (should succeed)
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "pr", "default", "--force"])
        .assert()
        .success();

    let content = fs::read_to_string(&pr_template_path).unwrap();
    assert!(!content.is_empty());
}

#[test]
fn test_pr_add_invalid_type() {
    let _temp_dir = setup_test_env();
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.args(&["add", "pr", "invalid-template"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not Found"));
}

#[test]
fn test_pr_add_unknown_argument() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "pr", "--unknown"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unexpected argument '--unknown' found",
        ));
}

#[test]
fn test_pr_add_valid_and_invalid_template() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    // Add both a valid and an invalid template in a single command
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "pr", "default", "invalid-template"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Not Found").or(predicate::str::contains("invalid-template")),
        );

    // The valid template should still be added
    assert_file_exists(&temp_path.join(".github/pull_request_template.md"));
}

#[test]
fn test_pr_add_default_with_output_without_ext() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "pr", "default", "-o", "default"])
        .assert()
        .success()
        .stdout(predicate::str::contains("default.md - has been added."));
}

#[test]
fn test_pr_add_default_with_output_with_ext() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "pr", "default", "-o", "default.md"])
        .assert()
        .success()
        .stdout(predicate::str::contains("default.md - has been added."));
}

#[test]
fn test_pr_add_uneven_templates_and_outputs() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    create_git_repo(&temp_path);

    // Provide two templates but only one output file name
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["add", "pr", "default", "default", "-o", "file1.md"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "The number of templates and output file names must match.",
        ));
}
// --------     LIST COMMAND TESTS     --------

#[test]
fn test_pr_list() {
    let _temp_dir = setup_test_env();
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.args(&["list", "prs"])
        .assert()
        .success()
        .stdout(predicate::str::contains("default.md"));
}

// --------     PREVIEW COMMAND TESTS     --------

#[test]
fn test_pr_preview_single() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["preview", "pr", "default"])
        .assert()
        .success()
        .stdout(predicate::str::is_match(".+").unwrap());
}

#[test]
fn test_pr_preview_invalid_id() {
    let temp_dir = setup_test_env();
    let temp_path = temp_dir.path().to_path_buf();

    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.current_dir(&temp_path);
    cmd.args(&["preview", "pr", "not-a-template"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("No issue template specified")
                .or(predicate::str::contains("Not Found")),
        );
}

// --------     HELP COMMAND TEST     --------
#[test]
fn test_pr_help_command() {
    let _temp_dir = setup_test_env();

    // add pr help
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.args(&["add", "pr", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Add a pull request template"))
        .stdout(predicate::str::contains("Usage: gitforge add pr-template"))
        .stdout(predicate::str::contains("--dir"))
        .stdout(predicate::str::contains("--force"));

    // list pr help
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.args(&["list", "pr", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("List available pull request templates"))
        .stdout(predicate::str::contains("Usage: gitforge list pr"));

    // preview pr help
    let mut cmd = AssertCommand::cargo_bin("gitforge").unwrap();
    cmd.args(&["preview", "pr", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Preview a pull request template"))
        .stdout(predicate::str::contains("Usage: gitforge preview pr"));
}
