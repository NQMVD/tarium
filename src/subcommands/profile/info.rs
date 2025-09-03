use colored::Colorize;
use libarov::{
    config::{filters::ProfileParameters as _, structs::Profile},
    iter_ext::IterExt as _,
};

pub fn info(profile: &Profile, active: bool) {
    println!(
        "{}{}
        \r  Output directory:   {}{}
        \r  Mods:               {}\n",
        if active {
            profile.name.bold().italic()
        } else {
            profile.name.bold()
        },
        if active { " *" } else { "" },
        profile.output_dir.display().to_string().blue().underline(),
        profile
            .filters
            .game_versions()
            .map(|v| format!(
                "\n  SPT Version:  {}",
                v.iter()
                    .map(AsRef::as_ref)
                    .map(Colorize::green)
                    .display(", ")
            ))
            .unwrap_or_default(),
        profile.mods.len().to_string().yellow(),
    );
    dbg!(&profile.filters);
}
