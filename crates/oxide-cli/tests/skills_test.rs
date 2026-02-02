//! 技能系统集成测试

#[tokio::test]
async fn test_custom_skills_loading() {
    // 确保测试前技能目录存在
    let skills_dir = oxide_core::config::oxide_home()
        .unwrap()
        .join("skills");
    std::fs::create_dir_all(&skills_dir).ok();

    // 创建一个测试技能
    let test_skill_dir = skills_dir.join("integration-test-skill");
    std::fs::create_dir_all(&test_skill_dir).ok();
    std::fs::write(
        test_skill_dir.join("SKILL.md"),
        "---\nname: integration-test-skill\ndescription: An integration test skill\n---\n\n# Test\n",
    ).ok();

    // 测试加载
    let registry = oxide_tools::skill::create_registry_with_custom()
        .await
        .expect("Failed to create registry");

    // 检查内置技能
    assert!(
        registry.get("commit").await.is_some(),
        "commit skill should be registered"
    );
    assert!(
        registry.get("feature-dev").await.is_some(),
        "feature-dev skill should be registered"
    );

    // 检查自定义技能
    assert!(
        registry.get("integration-test-skill").await.is_some(),
        "custom skill should be loaded"
    );

    // 验证技能数量
    let all_skills = registry.list_metadata().await;
    assert!(
        all_skills.len() >= 3,
        "should have at least 3 skills (2 builtin + 1 custom)"
    );

    // 验证技能元数据
    let custom_skills = registry.list_custom().await;
    assert!(
        !custom_skills.is_empty(),
        "should have at least one custom skill"
    );

    // 打印所有技能
    println!("\n=== 已加载的技能 ===");
    let all_skills = registry.list_metadata().await;
    for skill in &all_skills {
        println!("  - [{}] {}: {}", skill.kind, skill.name, skill.description);
    }

    // 清理
    std::fs::remove_dir_all(&test_skill_dir).ok();
}

#[tokio::test]
async fn test_command_registry_includes_custom_skills() {
    // 确保测试前技能目录存在
    let skills_dir = oxide_core::config::oxide_home()
        .unwrap()
        .join("skills");
    std::fs::create_dir_all(&skills_dir).ok();

    // 创建命令注册表
    let command_registry = oxide_cli::commands::create_registry().await;

    // 检查 /skills 命令是否存在
    let skills_cmd = command_registry.get("skills");
    assert!(skills_cmd.is_some(), "/skills command should be registered");

    // 检查 hello-world 技能是否作为命令被注册
    let hello_world_cmd = command_registry.get("hello-world");
    assert!(
        hello_world_cmd.is_some(),
        "hello-world skill should be registered as a command"
    );

    println!("\n=== 命令注册表中的技能命令 ===");
    // 列出所有命令名称
    let all_commands = command_registry.command_names();
    let skill_commands: Vec<_> = all_commands
        .iter()
        .filter(|n| **n != "skills" && (**n == "commit" || **n == "feature-dev" || **n == "hello-world"))
        .collect();

    println!("技能命令: {:?}", skill_commands);
}

#[tokio::test]
async fn test_project_skills_loading() {
    // 确保全局技能目录存在
    let global_skills_dir = oxide_core::config::oxide_home()
        .unwrap()
        .join("skills");
    std::fs::create_dir_all(&global_skills_dir).ok();

    // 创建一个全局技能
    let global_skill_dir = global_skills_dir.join("global-test-skill");
    std::fs::create_dir_all(&global_skill_dir).ok();
    std::fs::write(
        global_skill_dir.join("SKILL.md"),
        "---\nname: global-test-skill\ndescription: A global test skill\n---\n\n# Global Test\n",
    ).ok();

    // 创建项目目录
    let temp_dir = std::env::temp_dir().join("oxide_project_skills_test");
    std::fs::create_dir_all(&temp_dir).ok();

    // 创建一个项目技能
    let project_skill_dir = temp_dir.join(".oxide").join("skills").join("project-test-skill");
    std::fs::create_dir_all(&project_skill_dir).ok();
    std::fs::write(
        project_skill_dir.join("SKILL.md"),
        "---\nname: project-test-skill\ndescription: A project test skill\n---\n\n# Project Test\n",
    ).ok();

    // 测试加载
    let registry = oxide_tools::skill::create_registry_with_custom_and_project(Some(temp_dir.clone()))
        .await
        .expect("Failed to create registry");

    // 检查全局技能
    assert!(
        registry.get("global-test-skill").await.is_some(),
        "global skill should be loaded"
    );

    // 检查项目技能
    assert!(
        registry.get("project-test-skill").await.is_some(),
        "project skill should be loaded"
    );

    // 打印所有技能
    println!("\n=== 已加载的技能（全局+项目） ===");
    let all_skills = registry.list_metadata().await;
    for skill in &all_skills {
        println!("  - [{}] {}: {}", skill.kind, skill.name, skill.description);
    }

    // 清理
    std::fs::remove_dir_all(&global_skill_dir).ok();
    std::fs::remove_dir_all(&temp_dir).ok();
}

#[tokio::test]
async fn test_current_project_skills() {
    // 测试当前项目的技能加载
    let project_dir = std::path::PathBuf::from("/Users/c.chen/dev/oxide");

    let registry = oxide_tools::skill::create_registry_with_custom_and_project(Some(project_dir))
        .await
        .expect("Failed to create registry");

    println!("\n=== 当前项目的技能 ===");
    let _all_skills = registry.list_metadata().await;

    let builtin = registry.list_builtin().await;
    println!("内置技能 ({}):", builtin.len());
    for s in &builtin {
        println!("  - {}", s.name);
    }

    let custom = registry.list_custom().await;
    println!("\n自定义技能 ({}):", custom.len());
    for s in &custom {
        println!("  - {}: {}", s.name, s.description);
    }
}
