
dir structure:

SPT
- BepInEx
    - plugins
        - NAME/ | NAME.dll

- user
    - mods
        - NAME
            - package.json
                - "name": "SPT-DynamicMaps",
                - "version": "0.5.7",
                - "sptVersion": "~3.11",
                - "sptVersion": ">=3.11.2 <3.12.0",
                - "dependencies": { } not in use?

- BAK or .old ?
    - BepInEx ...
    - user ...


changelog and versions:
https://github.com/sp-tarkov/build/releases


mods github releases, name and assets

Hub Name + Version:
- Github ID
- Release Name
- Asset Zip Name


Dynamic Maps 0.5.7:
- acidphantasm/SPT-DynamicMaps
- 0.5.7
- DynamicMaps-0.5.7-ba475092.zip


Artem 2.1.2:
- no github lol

MOAR + Bagels - Ultra lite spawn mod 3.1.6:
- Andrewgdewar/MOAR-Client
- no releases on github

Algorithmic Level Progression 5.5.0-Beta:
- Andrewgdewar/AlgorithmicLevelProgression
- ALP 5.5.0-RC1
- AlgorithmicLevelProgression-5.5.0-RC1.zip

Better Rear Sights 1.6.2:
- peinwastaken/PeinBetterRearSights
- Release 1.6.2
- BetterRearSights.zip

MoreCheckmarks 1.5.17:
- TommySoucy/MoreCheckmarks
- v1.5.17 Small Fixes - 35392
- MoreCheckmarks-1.5.17-for-3.11.1-35392.7z

ODT's Item Info - 3.11 Update & Added Colored Name 1.1.0:
- thuynguyentrungdang/ODT-ItemInfo
- Aug242025-3.11
- Aug242025_311.zip

Item Info 4.4.0: [3.10]
- odt1/ODT-ItemInfo
- ItemInfo 4.4.0
- odt-iteminfo.zip

Two Slot Extended Mags 1.0.5b:
- gndworks/spt-2-slot-mags
- 1.0.5b
- platinum-twoslotextendedmags.zip

Colored Tracers 1.1.1:
- Szonszczyk/Colored-Tracers
- Version 1.1.1 for SPT 3.11.*
- Szonszczyk-ColoredTracers.zip

Weapon Customizer 2.0.1:
- tyfon7/WeaponCustomizer
- v2.0.1
- Tyfon-WeaponCustomizer-2.0.1.zip

SAIN - Solarint's AI Modifications - Full AI Combat System Replacement 4.1.3:
- Solarint/SAIN
- SAIN 4.1.3 for SPT 3.11.x
- SAIN-4.1.3.zip

Server Value Modifier [SVM] 1.11.1:
- GhostFenixx/SVM
- SVM 1.11.1
- SVM.Server.Value.Modifier1.11.1.zip

Looting Bots 1.6.0 (SPT 3.11):
- Skwizzy/SPT-LootingBots
- Version 1.6.0 (SPT 3.11)
- Skwizzy-LootingBots-1.6.0.zip

Virtual's Custom Quest Loader 2.0.4:
- VirtualAE/Virtuals-Custom-Quest-Loader
- SPT 3.11 v2.0.4
- VCQL-2.0.4.zip





parse the versions in the release name and assume that 3.11.* or 3.10.* is the SPT version its for _shrug_

also find a 7z crate...

add a black list for shit like SVM who needs another app to setup fully?!






### First Startup

You can have your own set of mods in what is called a 'profile'.

- Create a new profile by running `ferium profile create` and entering the details for your profile.
  - Then, add your mods using `ferium add`.
  - Finally, download your mods using `ferium upgrade`.

### Automatically Import Mods

```bash
ferium scan
```

This command scans a directory with mods, and attempts to add them to your profile.

The directory defaults to your profile's output directory. Some mods are available on both Modrinth and CurseForge; ferium will prefer Modrinth by default, but you can choose CurseForge instead using the `--platform` flag.

As long as you ensure the mods in the directory match the configured mod loader and Minecraft version, they should all add properly. Some mods might require some [additional tuning](#check-overrides). You can also bypass the compatibility checks using the `--force` flag.

### Manually Adding Mods

> [!TIP]
> You can specify multiple identifiers to add multiple mods at once

#### GitHub
```bash
ferium add owner/name
```
`owner` is the username of the owner of the repository and `name` is the name of the repository, both are case-insensitive (e.g. [Sodium's repository](https://github.com/CaffeineMC/sodium) has the id `CaffeineMC/sodium`). You can find these at the top left of the repository's page.  
So to add [Sodium](https://github.com/CaffeineMC/sodium), you should run `ferium add CaffeineMC/sodium`.

> [!IMPORTANT]
> The GitHub repository needs to upload JAR files to their _Releases_ for ferium to download, or else it will refuse to be added.

#### User Mods

If you want to use files that are not downloadable by ferium, place them in a subfolder called `user` in the output directory. Files here will be copied to the output directory when upgrading.

> [!NOTE]
> Profiles using Quilt will not copy their user mods, this is because Quilt automatically loads mods from nested directories (such as the user folder) since version `0.18.1-beta.3`.
  

mod files will be deleted on upgrade!!! there is a simple backup feature already, but its gonna do it everytime now