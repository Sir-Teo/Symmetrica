//! End-to-end tests for documentation completeness and quality.
//!
//! These tests verify that required documentation files exist and meet
//! quality standards for a production-ready 1.0 release.

use std::path::Path;

/// Test that all required root-level documentation files exist
#[test]
fn test_required_docs_exist() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    let required_files = vec![
        "README.md",
        "CHANGELOG.md",
        "LICENSE-MIT",
        "LICENSE-APACHE",
        "SECURITY.md",
        "API_STABILITY.md",
        "MIGRATION.md",
        "COVERAGE_IMPROVEMENTS.md",
    ];

    for file in required_files {
        let path = workspace_root.join(file);
        assert!(path.exists(), "Required documentation file missing: {}", file);

        // Verify file is not empty
        let metadata = std::fs::metadata(&path)
            .unwrap_or_else(|_| panic!("Cannot read metadata for {}", file));
        assert!(metadata.len() != 0, "Documentation file is empty: {}", file);
    }
}

/// Test that SECURITY.md contains required sections
#[test]
fn test_security_md_completeness() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    let security_path = workspace_root.join("SECURITY.md");
    let content =
        std::fs::read_to_string(&security_path).expect("SECURITY.md should exist and be readable");

    // Required sections for a complete security policy
    let required_sections = vec![
        "Supported Versions",
        "Reporting a Vulnerability",
        "Security Best Practices",
        "Disclosure Policy",
    ];

    for section in required_sections {
        assert!(content.contains(section), "SECURITY.md missing required section: {}", section);
    }

    // Should mention contact method
    assert!(
        content.contains("security@") || content.contains("Security Advisories"),
        "SECURITY.md should provide a security contact method"
    );
}

/// Test that README.md contains essential sections
#[test]
fn test_readme_completeness() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    let readme_path = workspace_root.join("README.md");
    let content =
        std::fs::read_to_string(&readme_path).expect("README.md should exist and be readable");

    let required_sections = vec![
        "Symmetrica",
        "Key Features",
        "Quickstart",
        "Usage Examples",
        "Contributing",
        "Licensing",
    ];

    for section in required_sections {
        assert!(content.contains(section), "README.md missing required section: {}", section);
    }

    // Should have CI badge
    assert!(content.contains("[![CI]"), "README.md should include CI status badge");
}

/// Test that CHANGELOG.md follows Keep a Changelog format
#[test]
fn test_changelog_format() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    let changelog_path = workspace_root.join("CHANGELOG.md");
    let content = std::fs::read_to_string(&changelog_path)
        .expect("CHANGELOG.md should exist and be readable");

    // Should follow Keep a Changelog format
    assert!(content.contains("# Changelog"), "CHANGELOG.md should have a title");
    assert!(content.contains("[Unreleased]"), "CHANGELOG.md should have an Unreleased section");
    assert!(
        content.contains("keepachangelog.com"),
        "CHANGELOG.md should reference Keep a Changelog"
    );
}

/// Test that API_STABILITY.md defines clear guarantees
#[test]
fn test_api_stability_guarantees() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    let api_path = workspace_root.join("API_STABILITY.md");
    let content =
        std::fs::read_to_string(&api_path).expect("API_STABILITY.md should exist and be readable");

    let required_sections = vec![
        "Semantic Versioning",
        "1.0 Stability Guarantees",
        "Breaking Change Policy",
        "Mathematical Correctness Guarantees",
    ];

    for section in required_sections {
        assert!(
            content.contains(section),
            "API_STABILITY.md missing required section: {}",
            section
        );
    }

    // Should reference semver.org
    assert!(
        content.contains("semver.org"),
        "API_STABILITY.md should reference Semantic Versioning"
    );
}

/// Test that all module documentation files exist
#[test]
fn test_module_docs_exist() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    let docs_dir = workspace_root.join("docs");

    let required_docs = vec![
        // Core modules
        "expr_core.md",
        "arith.md",
        "simplify.md",
        "pattern.md",
        // Mathematical modules
        "calculus.md",
        "polys.md",
        "matrix.md",
        "solver.md",
        "assumptions.md",
        // I/O and applications
        "io.md",
        "evalf.md",
        "plot.md",
        "cli.md",
        "api.md",
        "wasm.md",
        // Quality assurance
        "fuzzing.md",
        "property_testing.md",
        "differential_testing.md",
        "benchmarking.md",
        // Architecture
        "roadmap.md",
        "skeleton.md",
        "research.md",
    ];

    for doc in required_docs {
        let path = docs_dir.join(doc);
        assert!(path.exists(), "Required module documentation missing: docs/{}", doc);
    }
}

/// Test that MIGRATION.md contains required sections
#[test]
fn test_migration_guide_completeness() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    let migration_path = workspace_root.join("MIGRATION.md");
    let content = std::fs::read_to_string(&migration_path)
        .expect("MIGRATION.md should exist and be readable");

    // Required sections for a complete migration guide
    let required_sections = vec![
        "Migration Guide",
        "What's New in 1.0.0",
        "API Changes",
        "Migration Checklist",
        "Troubleshooting",
    ];

    for section in required_sections {
        assert!(content.contains(section), "MIGRATION.md missing required section: {}", section);
    }

    // Should mention version numbers
    assert!(
        content.contains("0.1") && content.contains("1.0"),
        "MIGRATION.md should reference both 0.1.x and 1.0.0 versions"
    );

    // Should have code examples
    assert!(content.contains("```rust"), "MIGRATION.md should include Rust code examples");
}

/// Test that GitHub templates exist
#[test]
fn test_github_templates_exist() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    let github_dir = workspace_root.join(".github");

    let required_templates = vec![
        "PULL_REQUEST_TEMPLATE.md",
        "CODEOWNERS",
        "ISSUE_TEMPLATE/bug_report.md",
        "ISSUE_TEMPLATE/feature_request.md",
        "ISSUE_TEMPLATE/config.yml",
    ];

    for template in required_templates {
        let path = github_dir.join(template);
        assert!(path.exists(), "Required GitHub template missing: .github/{}", template);
    }
}

/// Test that CI workflows exist and are properly configured
#[test]
fn test_ci_workflows_exist() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    let workflows_dir = workspace_root.join(".github/workflows");

    let required_workflows = vec!["ci.yml", "fuzz.yml", "pages.yml"];

    for workflow in required_workflows {
        let path = workflows_dir.join(workflow);
        assert!(path.exists(), "Required CI workflow missing: .github/workflows/{}", workflow);

        // Verify workflow is not empty
        let content = std::fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("Cannot read workflow: {}", workflow));
        assert!(!content.is_empty(), "CI workflow is empty: {}", workflow);
    }
}

/// Test that the main CI workflow includes all required checks
#[test]
fn test_ci_workflow_completeness() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    let ci_path = workspace_root.join(".github/workflows/ci.yml");
    let content = std::fs::read_to_string(&ci_path).expect("ci.yml should exist and be readable");

    let required_checks =
        vec!["cargo fmt", "cargo clippy", "cargo test", "cargo doc", "cargo audit", "cargo deny"];

    for check in required_checks {
        assert!(content.contains(check), "CI workflow missing required check: {}", check);
    }
}

/// Test that license files are properly formatted
#[test]
fn test_license_files_valid() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();

    // Check MIT license
    let mit_path = workspace_root.join("LICENSE-MIT");
    let mit_content =
        std::fs::read_to_string(&mit_path).expect("LICENSE-MIT should exist and be readable");
    assert!(
        mit_content.contains("MIT License") || mit_content.contains("Permission is hereby granted"),
        "LICENSE-MIT should contain MIT license text"
    );

    // Check Apache license
    let apache_path = workspace_root.join("LICENSE-APACHE");
    let apache_content =
        std::fs::read_to_string(&apache_path).expect("LICENSE-APACHE should exist and be readable");
    assert!(
        apache_content.contains("Apache License") && apache_content.contains("Version 2.0"),
        "LICENSE-APACHE should contain Apache 2.0 license text"
    );
}
