# Managing Mod States (Enable/Disable)

Tarium now supports enabling and disabling mods without completely removing them from your profile. This is similar to functionality found in CurseForge and other mod managers.

## Enabling and Disabling Mods

To disable mods (move files to disabled folder without removing from profile):
```powershell
tarium.exe disable [mod_names...]
```

To enable previously disabled mods:
```powershell
tarium.exe enable [mod_names...]
```

Examples:
```powershell
# Disable specific mods by name
tarium.exe disable SAIN SPT-Waypoints

# Disable mods interactively (select from list)
tarium.exe disable

# Enable specific mods
tarium.exe enable SAIN

# Enable mods interactively
tarium.exe enable
```

## How It Works

- **Disabling mods**: When you disable a mod, Tarium moves all files associated with that mod from your active SPT directories to a `disabled-mods` folder within your profile's output directory. The mod remains in your profile but is marked as disabled.

- **Enabling mods**: When you enable a mod, Tarium moves all files associated with that mod from `disabled-mods` folder back to their original locations in your SPT directories. The mod is marked as enabled.

- **Persistence**: The enabled/disabled state of each mod is saved in your profile configuration and persists across sessions.

- **File Tracking**: Tarium automatically tracks which files belong to each mod by analyzing archive contents and monitoring file patterns during installation.

## Visual Indicators

When you run `tarium.exe list`, you'll see status indicators for each mod:
- `✓` - Mod is currently enabled
- `✗` - Mod is currently disabled

## Directory Structure

When using enable/disable functionality, Tarium creates the following structure:
```
your_spt_folder/
├── BepInEx/
│   └── plugins/
├── user/
│   └── mods/
├── disabled-mods/
│   ├── ModName1/
│   │   ├── file1.dll
│   │   └── config.json
│   └── ModName2/
│       └── plugin.dll
└── MODS/
```

## Important Notes

- **Safe Operation**: The enable/disable operations simply move files between directories. No files are deleted, so you can safely enable/disable mods without losing data.

- **Compatibility**: This feature works with both ZIP and 7z archives that Tarium supports.

- **Automatic Tracking**: When you add new mods or run upgrades, Tarium automatically tracks which files belong to each mod for proper enable/disable functionality.

- **No Restart Required**: You can enable/disable mods while SPT is not running, then start the game to see changes take effect.