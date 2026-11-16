use std::fs;
use editor_core::{Cursor, Editor};
use extensions::{ExtensionCapability, ExtensionManifest, SandboxSecurity};
use filesystem::{FileSystem, LineEnding};
use git::GitDiff;
use search::{SearchEngine, SearchQuery};
use settings::{EditorSettings, VsCodeImporter};

#[test]
fn test_end_to_end_editing_and_persistence() {
    let temp_dir = std::env::temp_dir().join("oxide_e2e_test");
    let file_path = temp_dir.join("main.rs");

    // 1. Initialize editor with initial template
    let mut ed = Editor::new();
    ed.insert_text("fn main() {\n");
    ed.insert_text("    let x = 10;\n");
    ed.insert_text("}\n");

    assert_eq!(ed.buffer.len_lines(), 4);
    assert!(ed.buffer.is_dirty());

    // 2. Multi-cursor editing
    // Add prefix "// " to lines 1 and 2
    let line1_start = ed.buffer.line_to_char(0);
    let line2_start = ed.buffer.line_to_char(1);
    ed.selections.set_cursors(vec![Cursor::new(line1_start), Cursor::new(line2_start)]);
    ed.insert_text("// ");

    assert!(ed.buffer.to_string().starts_with("// fn main() {\n//     let x = 10;"));

    // 3. Test Undo restores exactly to before the edit
    ed.undo();
    assert_eq!(
        ed.buffer.to_string(),
        "fn main() {\n    let x = 10;\n}\n"
    );

    // 4. Test Redo reapplies the edit
    ed.redo();
    assert!(ed.buffer.to_string().starts_with("// fn main() {\n//     let x = 10;"));

    // 5. Test Atomic Save
    FileSystem::write_file_atomic(&file_path, &ed.buffer.to_string(), LineEnding::Lf).unwrap();
    ed.buffer.mark_saved();
    assert!(!ed.buffer.is_dirty());

    // 6. Test File Reading & Encoding
    let (read_content, line_ending) = FileSystem::read_file(&file_path).unwrap();
    assert_eq!(read_content, ed.buffer.to_string());
    assert_eq!(line_ending, LineEnding::Lf);

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_gutter_diff_and_search_integration() {
    let original = "fn calculate() -> i32 {\n    let a = 1;\n    let b = 2;\n    a + b\n}\n";
    let modified = "fn calculate() -> i32 {\n    let a = 10;\n    let b = 2;\n    let c = 3;\n    a + b + c\n}\n";

    // 1. Verify Gutter Markers
    let markers = GitDiff::compute_gutter_markers(original, modified);
    assert!(!markers.is_empty());
    // Line 1 ("let a = 10;") was modified
    assert!(markers.contains_key(&1));

    // 2. Verify Search Engine
    let temp_dir = std::env::temp_dir().join("oxide_search_test");
    let file1 = temp_dir.join("calc.rs");
    let file2 = temp_dir.join("helper.rs");
    FileSystem::write_file_atomic(&file1, modified, LineEnding::Lf).unwrap();
    FileSystem::write_file_atomic(&file2, "pub fn helper() { let c = 42; }", LineEnding::Lf).unwrap();

    let query = SearchQuery {
        query: "let c".to_string(),
        is_regex: false,
        case_sensitive: true,
        whole_word: false,
        max_results: 10,
    };

    let results = SearchEngine::search(std::slice::from_ref(&temp_dir), &query);
    assert_eq!(results.len(), 2);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_vscode_import_and_extension_security() {
    // 1. VS Code Settings Import
    let vscode_json = r#"{
        "editor.fontSize": 15,
        "editor.tabSize": 2,
        "editor.insertSpaces": true,
        "workbench.colorTheme": "One Dark Pro"
    }"#;

    let defaults = EditorSettings::default();
    let imported = VsCodeImporter::import_settings_json(vscode_json, &defaults);
    assert_eq!(imported.font_size, 15.0);
    assert_eq!(imported.tab_size, 2);
    assert_eq!(imported.theme, "One Dark Pro");

    // 2. Extension Manifest & Sandbox Security
    let manifest_json = r#"{
        "name": "git-blame-plugin",
        "version": "1.0.0",
        "publisher": "community",
        "description": "Git blame provider",
        "activation_events": ["onCommand:git.blame"],
        "capabilities": ["FilesystemRead"],
        "contributes": {
            "commands": [],
            "languages": [],
            "themes": []
        }
    }"#;

    let manifest = ExtensionManifest::from_json(manifest_json).unwrap();
    assert_eq!(manifest.name, "git-blame-plugin");

    let sandbox = SandboxSecurity::from_manifest(&manifest);
    assert!(sandbox.check_permission(ExtensionCapability::FilesystemRead).is_ok());
    assert!(sandbox.check_permission(ExtensionCapability::TerminalExecution).is_err());
}
