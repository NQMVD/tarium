#![expect(clippy::unwrap_used)]

use crate::{
    actual_main,
    cli::{FilterArguments, ProfileSubCommands, SubCommands, Tarium},
};
use std::{
    env::current_dir,
    fs::{copy, create_dir_all},
    path::PathBuf,
};

const DEFAULT: Tarium = Tarium {
    subcommand: SubCommands::Profile { subcommand: None },
    threads: None,
    parallel_tasks: 10,
    github_token: None,
    config_file: None,
    verbosity: 2,
};

fn get_args(subcommand: SubCommands, config_file: Option<&str>) -> Tarium {
    let running = PathBuf::from(".")
        .join("tests")
        .join("configs")
        .join("running")
        .join(format!("{:X}.json", rand::random::<u8>()));
    let _ = create_dir_all(running.parent().unwrap());
    if let Some(config_file) = config_file {
        copy(format!("./tests/configs/{config_file}.json"), &running).unwrap();
    }
    Tarium {
        subcommand,
        config_file: Some(running),
        ..DEFAULT
    }
}

// TODO
// #[tokio::test(flavor = "multi_thread")]
// async fn arg_parse() {}

#[tokio::test(flavor = "multi_thread")]
async fn create_profile_no_profiles_to_import() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Profile {
                subcommand: Some(ProfileSubCommands::Create {
                    // There should be no other profiles to import mods from
                    import: Some(None),
                    game_version: vec!["1.21.4".to_owned()],
                    name: Some("Test Profile".to_owned()),
                    output_dir: Some(current_dir().unwrap().join("tests").join("mods")),
                })
            },
            None,
        ))
        .await,
        Err(_),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn create_profile_rel_dir() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Profile {
                subcommand: Some(ProfileSubCommands::Create {
                    // There should be no other profiles to import mods from
                    import: Some(None),
                    game_version: vec!["1.21.4".to_owned()],
                    name: Some("Test Profile".to_owned()),
                    output_dir: Some(PathBuf::from(".").join("tests").join("mods")),
                })
            },
            None,
        ))
        .await,
        Err(_),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn create_profile_import_mods() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Profile {
                subcommand: Some(ProfileSubCommands::Create {
                    // There should be no other profiles to import mods from
                    import: Some(Some("Default Modded".to_owned())),
                    game_version: vec!["1.21.4".to_owned()],
                    name: Some("Test Profile".to_owned()),
                    output_dir: Some(current_dir().unwrap().join("tests").join("mods")),
                })
            },
            Some("one_profile_full"),
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn create_profile_existing_name() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Profile {
                subcommand: Some(ProfileSubCommands::Create {
                    import: None,
                    game_version: vec!["1.21.4".to_owned()],
                    name: Some("Default Modded".to_owned()),
                    output_dir: Some(current_dir().unwrap().join("tests").join("mods"))
                })
            },
            None,
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn create_profile() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Profile {
                subcommand: Some(ProfileSubCommands::Create {
                    import: None,
                    game_version: vec!["1.21.4".to_owned()],
                    name: Some("Test Profile".to_owned()),
                    output_dir: Some(current_dir().unwrap().join("tests").join("mods"))
                })
            },
            None,
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn add_modrinth() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Add {
                identifiers: vec!["starlight".to_owned()],
                force: false,
                pin: None,
                filters: FilterArguments::default(),
            },
            Some("empty_profile"),
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn add_curseforge() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Add {
                identifiers: vec!["591388".to_owned()],
                force: false,
                pin: None,
                filters: FilterArguments::default(),
            },
            Some("empty_profile"),
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn add_github() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Add {
                identifiers: vec!["CaffeineMC/sodium".to_owned()],
                force: false,
                pin: None,
                filters: FilterArguments::default(),
            },
            Some("empty_profile"),
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn add_all() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Add {
                identifiers: vec![
                    "starlight".to_owned(),
                    "591388".to_owned(),
                    "CaffeineMC/sodium".to_owned()
                ],
                force: false,
                pin: None,
                filters: FilterArguments::default(),
            },
            Some("empty_profile"),
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn already_added() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Add {
                identifiers: vec![
                    "starlight".to_owned(),
                    "591388".to_owned(),
                    "CaffeineMC/sodium".to_owned()
                ],
                force: false,
                pin: None,
                filters: FilterArguments::default(),
            },
            Some("one_profile_full"),
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn list_no_profile() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::List {
                verbose: false,
                markdown: false
            },
            Some("empty"),
        ))
        .await,
        Err(_),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn list_empty_profile() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::List {
                verbose: false,
                markdown: false
            },
            Some("empty_profile"),
        ))
        .await,
        Err(_),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn list() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::List {
                verbose: false,
                markdown: false
            },
            Some("one_profile_full"),
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn list_verbose() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::List {
                verbose: true,
                markdown: false
            },
            Some("one_profile_full"),
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn list_markdown() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::List {
                verbose: true,
                markdown: true
            },
            Some("one_profile_full"),
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn list_profiles() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Profiles,
            Some("two_profiles_one_empty"),
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn upgrade() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Upgrade { local_only: false },
            Some("one_profile_full")
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn upgrade_local_only() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Upgrade { local_only: true },
            Some("one_profile_full")
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn profile_switch() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Profile {
                subcommand: Some(ProfileSubCommands::Switch {
                    profile_name: Some("Profile Two".to_owned())
                })
            },
            Some("two_profiles_one_empty")
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn remove_fail() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Remove {
                mod_names: vec![
                    "starlght (fabric)".to_owned(),
                    "incendum".to_owned(),
                    "sodum".to_owned(),
                ]
            },
            Some("two_profiles_one_empty")
        ))
        .await,
        Err(_),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn remove_name() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Remove {
                mod_names: vec![
                    "starlight (fabric)".to_owned(),
                    "incendium".to_owned(),
                    "sodium".to_owned(),
                ]
            },
            Some("two_profiles_one_empty")
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn remove_id() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Remove {
                mod_names: vec![
                    "H8CaAYZC".to_owned(),
                    "591388".to_owned(),
                    "caffeinemc/sodium".to_owned(),
                ]
            },
            Some("two_profiles_one_empty")
        ))
        .await,
        Ok(()),
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn remove_slug() {
    // Load the slugs into the config first
    let mut args = get_args(
        SubCommands::List {
            verbose: true,
            markdown: false,
        },
        Some("two_profiles_one_empty"),
    );
    assert!(matches!(actual_main(args.clone()).await, Ok(())));

    args.subcommand = SubCommands::Remove {
        mod_names: vec![
            "starlight".to_owned(),
            "incendium".to_owned(),
            "sodium".to_owned(),
        ],
    };
    assert!(matches!(actual_main(args).await, Ok(())));
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_profile() {
    assert!(matches!(
        actual_main(get_args(
            SubCommands::Profile {
                subcommand: Some(ProfileSubCommands::Delete {
                    profile_name: Some("Profile Two".to_owned()),
                    switch_to: None
                })
            },
            Some("two_profiles_one_empty")
        ))
        .await,
        Ok(()),
    ));
}
