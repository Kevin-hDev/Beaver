use serde_json::json;
use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::agent_turn_contract::{NewUserTurnInput, SkillReference, TurnAttachmentInput};

use super::conversation_input::{self, ConversationInputErrorKind};

const TEST_KEY: [u8; 32] = [31; 32];

fn input(files: Vec<TurnAttachmentInput>, skills: Vec<SkillReference>) -> NewUserTurnInput {
    NewUserTurnInput {
        content: "question".to_string(),
        files,
        skills,
    }
}

fn thumbnail(name: &str, mime_type: &str, base64: String) -> TurnAttachmentInput {
    TurnAttachmentInput {
        name: name.to_string(),
        path: String::new(),
        mime_type: mime_type.to_string(),
        size: 0,
        thumbnail: Some(format!("data:{mime_type};base64,{base64}")),
        access_grant: None,
    }
}

#[test]
fn sensitive_nested_fields_cannot_be_deserialized() {
    assert!(serde_json::from_value::<NewUserTurnInput>(json!({
        "content": "question",
        "files": [],
        "skills": [{"id": "safe", "content": "forged"}],
        "continuation": {"type": "local_replay"}
    }))
    .is_err());
}

#[tokio::test]
async fn collection_limits_are_rejected_without_truncation() {
    let images = (0..9)
        .map(|index| thumbnail(&format!("{index}.png"), "image/png", valid_png_base64()))
        .collect();
    let error = conversation_input::resolve_with_key(input(images, vec![]), &TEST_KEY)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ConversationInputErrorKind::Limit);

    let skills = (0..9)
        .map(|index| SkillReference {
            id: format!("local:skill:{index}"),
            name: None,
        })
        .collect();
    let error = conversation_input::resolve_with_key(input(vec![], skills), &TEST_KEY)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ConversationInputErrorKind::Limit);
}

#[tokio::test]
async fn image_thumbnail_is_validated_by_real_content() {
    let image = thumbnail("image.png", "image/jpeg", valid_png_base64());
    let error = conversation_input::resolve_with_key(input(vec![image], vec![]), &TEST_KEY)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ConversationInputErrorKind::Type);

    let invalid = thumbnail("image.png", "image/png", "@@@@".to_string());
    let error = conversation_input::resolve_with_key(input(vec![invalid], vec![]), &TEST_KEY)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ConversationInputErrorKind::Type);
}

#[tokio::test]
async fn vault_key_is_loaded_only_when_a_path_requires_a_grant() {
    let calls = Cell::new(0_usize);
    let text_only = conversation_input::resolve_with_key_source(input(vec![], vec![]), || {
        calls.set(calls.get() + 1);
        Err("vault unavailable".to_string())
    })
    .await
    .unwrap();
    assert_eq!(text_only.user_content, "question");
    assert_eq!(calls.get(), 0);

    let inline = thumbnail("image.png", "image/png", valid_png_base64());
    conversation_input::resolve_with_key_source(input(vec![inline], vec![]), || {
        calls.set(calls.get() + 1);
        Err("vault unavailable".to_string())
    })
    .await
    .unwrap();
    assert_eq!(calls.get(), 0);

    let path_input = TurnAttachmentInput {
        name: "notes.txt".into(),
        path: "/tmp/notes.txt".into(),
        mime_type: "txt".into(),
        size: 1,
        thumbnail: None,
        access_grant: Some("v1.invalid".into()),
    };
    let error =
        conversation_input::resolve_with_key_source(input(vec![path_input], vec![]), || {
            calls.set(calls.get() + 1);
            Err("vault unavailable".to_string())
        })
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ConversationInputErrorKind::Unavailable);
    assert_eq!(calls.get(), 1);
}

#[tokio::test]
async fn public_errors_are_identical_and_never_include_paths() {
    let path = "/private/user/secret.txt";
    let missing = TurnAttachmentInput {
        name: "secret.txt".into(),
        path: path.into(),
        mime_type: "text/plain".into(),
        size: 1,
        thumbnail: None,
        access_grant: Some("v1.invalid".into()),
    };
    let grant = conversation_input::resolve_with_key(input(vec![missing], vec![]), &TEST_KEY)
        .await
        .unwrap_err();
    let invalid_type = conversation_input::resolve_with_key(
        input(
            vec![thumbnail(
                "bad.exe",
                "application/x-msdownload",
                valid_png_base64(),
            )],
            vec![],
        ),
        &TEST_KEY,
    )
    .await
    .unwrap_err();
    let limit = conversation_input::resolve_with_key(
        input(
            (0..9)
                .map(|index| thumbnail(&format!("{index}.png"), "image/png", valid_png_base64()))
                .collect(),
            vec![],
        ),
        &TEST_KEY,
    )
    .await
    .unwrap_err();

    assert_eq!(grant.public_code(), invalid_type.public_code());
    assert_eq!(grant.public_code(), limit.public_code());
    assert!(!grant.to_string().contains(path));
    assert!(!invalid_type.to_string().contains(path));
}

#[tokio::test]
async fn text_limits_are_exact_and_unicode_safe() {
    let dir = tempfile::tempdir().unwrap();
    let exact_path = dir.path().join("exact.txt");
    fs::write(&exact_path, "😀".repeat(120_000)).unwrap();
    let exact = path_attachment(&exact_path, "txt");
    let resolved = conversation_input::resolve_with_key(input(vec![exact], vec![]), &TEST_KEY)
        .await
        .unwrap();
    assert!(resolved.provider_content.contains(&"😀".repeat(32)));

    let overflow_path = dir.path().join("overflow.txt");
    fs::write(&overflow_path, "😀".repeat(120_001)).unwrap();
    let overflow = path_attachment(&overflow_path, "text/plain");
    let error = conversation_input::resolve_with_key(input(vec![overflow], vec![]), &TEST_KEY)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ConversationInputErrorKind::Limit);
}

#[tokio::test]
async fn total_text_budget_accepts_300000_and_rejects_the_next_character() {
    let dir = tempfile::tempdir().unwrap();
    let exact = text_files(dir.path(), [100_000, 100_000, 100_000]);
    let resolved = conversation_input::resolve_with_key(input(exact, vec![]), &TEST_KEY)
        .await
        .unwrap();
    assert_eq!(resolved.files.len(), 3);

    let overflow_dir = tempfile::tempdir().unwrap();
    let overflow = text_files(overflow_dir.path(), [100_000, 100_000, 100_001]);
    let error = conversation_input::resolve_with_key(input(overflow, vec![]), &TEST_KEY)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ConversationInputErrorKind::Limit);
}

#[tokio::test]
async fn text_mime_matrix_accepts_known_pairs_and_rejects_misleading_types() {
    let dir = tempfile::tempdir().unwrap();
    for (name, declared) in [
        ("source.rs", "rs"),
        ("plain.rs", "text/plain"),
        ("page.html", "text/html"),
        ("data.json", "application/json"),
    ] {
        let path = dir.path().join(name);
        fs::write(&path, "safe text").unwrap();
        let attachment = path_attachment(&path, declared);
        conversation_input::resolve_with_key(input(vec![attachment], vec![]), &TEST_KEY)
            .await
            .unwrap();
    }

    let misleading_path = dir.path().join("source-misleading.rs");
    fs::write(&misleading_path, "safe text").unwrap();
    let misleading = path_attachment(&misleading_path, "text/html");
    let error = conversation_input::resolve_with_key(input(vec![misleading], vec![]), &TEST_KEY)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ConversationInputErrorKind::Type);
}

#[tokio::test]
async fn verified_file_and_inline_image_converge() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("same.png");
    let bytes = valid_png_bytes();
    fs::write(&path, bytes).unwrap();
    let file = path_attachment(&path, "png");
    let inline = thumbnail("same-inline.png", "image/png", valid_png_base64());

    let from_file = conversation_input::resolve_with_key(input(vec![file], vec![]), &TEST_KEY)
        .await
        .unwrap();
    let from_inline = conversation_input::resolve_with_key(input(vec![inline], vec![]), &TEST_KEY)
        .await
        .unwrap();

    assert_eq!(from_file.images, from_inline.images);
}

#[tokio::test]
async fn duplicate_attachments_and_sixteenth_attachment_are_rejected() {
    let duplicate = thumbnail("same.png", "image/png", valid_png_base64());
    let error = conversation_input::resolve_with_key(
        input(vec![duplicate.clone(), duplicate], vec![]),
        &TEST_KEY,
    )
    .await
    .unwrap_err();
    assert_eq!(error.kind(), ConversationInputErrorKind::Invalid);

    let sixteen = (0..16)
        .map(|index| thumbnail(&format!("{index}.png"), "image/png", valid_png_base64()))
        .collect();
    let error = conversation_input::resolve_with_key(input(sixteen, vec![]), &TEST_KEY)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ConversationInputErrorKind::Limit);
}

#[tokio::test]
async fn image_byte_limit_is_checked_before_base64_decode() {
    let max_encoded = ((crate::services::llm::vision::MAX_IMAGE_BYTES).div_ceil(3)) * 4;
    let oversized = thumbnail("large.png", "image/png", "A".repeat(max_encoded + 1));
    let error = conversation_input::resolve_with_key(input(vec![oversized], vec![]), &TEST_KEY)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ConversationInputErrorKind::Limit);
}

#[tokio::test]
async fn exactly_twenty_mib_image_is_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("large.png");
    let mut bytes = vec![0_u8; crate::services::llm::vision::MAX_IMAGE_BYTES];
    bytes[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
    fs::write(&path, bytes).unwrap();
    let file = path_attachment(&path, "image/png");

    let resolved = conversation_input::resolve_with_key(input(vec![file], vec![]), &TEST_KEY)
        .await
        .unwrap();

    assert_eq!(resolved.images.len(), 1);
    assert_eq!(
        resolved.files[0].size as usize,
        crate::services::llm::vision::MAX_IMAGE_BYTES
    );
}

#[tokio::test]
async fn skill_references_reload_local_authority_and_are_bounded() {
    let installed = install_skills(8);
    let references = installed
        .iter()
        .map(|(_, id, name)| SkillReference {
            id: id.clone(),
            name: Some(name.clone()),
        })
        .collect();
    let resolved = conversation_input::resolve_with_key(input(vec![], references), &TEST_KEY)
        .await
        .unwrap();

    assert_eq!(resolved.skills.len(), 8);
    assert!(resolved
        .skills
        .iter()
        .all(|skill| skill.content.contains("trusted body")));
    for (root, _, _) in installed {
        fs::remove_dir_all(root).unwrap();
    }
}

#[tokio::test]
async fn duplicate_invalid_and_missing_skills_are_rejected() {
    let installed = install_skills(1);
    let id = installed[0].1.clone();
    let duplicate = vec![
        SkillReference {
            id: id.clone(),
            name: None,
        },
        SkillReference { id, name: None },
    ];
    let error = conversation_input::resolve_with_key(input(vec![], duplicate), &TEST_KEY)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ConversationInputErrorKind::Invalid);

    for invalid in ["../secret", "bad/id", "bad\ncontrol"] {
        let error = conversation_input::resolve_with_key(
            input(
                vec![],
                vec![SkillReference {
                    id: invalid.into(),
                    name: None,
                }],
            ),
            &TEST_KEY,
        )
        .await
        .unwrap_err();
        assert_eq!(error.kind(), ConversationInputErrorKind::Invalid);
    }
    let missing = SkillReference {
        id: "local:skill:000000000000000000000000".into(),
        name: None,
    };
    let error = conversation_input::resolve_with_key(input(vec![], vec![missing]), &TEST_KEY)
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ConversationInputErrorKind::Skill);
    fs::remove_dir_all(&installed[0].0).unwrap();
}

fn path_attachment(path: &Path, declared_type: &str) -> TurnAttachmentInput {
    let raw = path.to_string_lossy().to_string();
    let registered = crate::services::attachment_access::register_paths(
        std::slice::from_ref(&raw),
        &TEST_KEY,
        |_| true,
    )
    .unwrap();
    TurnAttachmentInput {
        name: path.file_name().unwrap().to_string_lossy().to_string(),
        path: registered[0].path.clone(),
        mime_type: declared_type.to_string(),
        size: registered[0].size,
        thumbnail: None,
        access_grant: Some(registered[0].access_grant.clone()),
    }
}

fn text_files<const N: usize>(directory: &Path, sizes: [usize; N]) -> Vec<TurnAttachmentInput> {
    sizes
        .into_iter()
        .enumerate()
        .map(|(index, size)| {
            let path = directory.join(format!("{index}.txt"));
            fs::write(&path, "a".repeat(size)).unwrap();
            path_attachment(&path, "txt")
        })
        .collect()
}

fn install_skills(count: usize) -> Vec<(PathBuf, String, String)> {
    let skills_root = crate::services::paths::data_dir().join("skills");
    fs::create_dir_all(&skills_root).unwrap();
    (0..count)
        .map(|index| {
            let name = format!("turn-input-{}-{index}", uuid::Uuid::new_v4());
            let root = skills_root.join(&name);
            fs::create_dir_all(&root).unwrap();
            fs::write(
                root.join("SKILL.md"),
                format!("---\nname: {name}\ndescription: test\n---\ntrusted body {index}"),
            )
            .unwrap();
            let id = super::skill_catalog::entries()
                .unwrap()
                .into_iter()
                .find(|entry| entry.info.name == name && entry.info.source == "local")
                .unwrap()
                .info
                .id;
            (root, id, name)
        })
        .collect()
}

fn valid_png_base64() -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(valid_png_bytes())
}

fn valid_png_bytes() -> [u8; 12] {
    [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 0]
}
