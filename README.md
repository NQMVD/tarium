# Tarium

[![rust badge](https://img.shields.io/static/v1?label=Made%20with&message=Rust&logo=rust&labelColor=e82833&color=b11522)](https://www.rust-lang.org)

Alright so... here's a mod manager i guess.

Disclaimer: it's a simple one, as of right now you still have to go to the mods page on the hub to get the ID.
but after you added them to your list, it's all just one command to download and install them. No need to extract, drag and drop them.

I'm still working on it, but in testing it works just fine with alot of mods for 3.11
The main limitation right now is that it only works with Github (the place where devs actually upload the mods) which means:
- no connection to the SPT hub -> no access to the description they have, only the github readme one which is mostly not even present
- because of no proper conventions sometimes no way to tell the SPT version the mod is for...
- HEAVY ratelimits for now, as i haven't registered the App on Github yet, we're talking 60 requests/h (i'll include some tips for that later, also this will change with both the registration and further improvements of the app).

> At the bottom of this is a todo list that i maintain while working on this, just so you get an idea of how the current state is.

This is a fork btw, i took the [ferium](https://github.com/gorilla-devs/ferium) tool as a base and swapped/ripped out the parts related to minecraft for their SPT counterparts. Mostly for the profile and github code it had, but also because i like terminal tools and LLMs work really well with Rust).
For now its just the terminal tool, but i already have plans to create a Wrapper around it, to give you guys a proper App you can click and drag around in. It will use a framework called eframe which in other cases showed crazy minimal resource usage, to not impact game performance.
That will have to wait tho, until the program is more polished and tested.

### How to install it:
> I **highly** recommend creating a copy of your SPT installation before you use tarium right now.
> Not because the program isn't safe, but it _is_ considered experimental and in such cases it's always advised to make a backup.

- Just download both files (tarium.exe and OPEN-CMD.bat) from [this](https://github.com/NQMVD/tarium/releases/tag/v0.1.0-alpha) page under "Assets" and place them both in your SPT folder, next to the SPT Launcher and Server exes.
- That's it.

> But if you know what you're doing, you can install it where-ever-you-want and run it from where-ever-you-want, that's why profiles have the SPT dir stored.

### How to use it:
 _looks more complicated than it is, trust me._

1. Open the terminal by opening the OPEN-CMD.bat file (just double click)

2. Create a profile with `tarium.exe profile create`
    - when the file explorer pops up, go to your SPT installation and click "Select" at the bottom right
    - there might be a warning in the terminal TODO
    - give it a name you like, not linked to your SPT profile btw
    - select the version youre playing on. DISCLAIMER: i only tried 3.11, it might not work with 3.10 or 3.9, let me know if thats the case.

3. Now start adding mods.
    - To add a mod, you need the github identifier of the mod, for example: `Solarint/SAIN`
        - the id is on the right side of the mod page on the SPT hub, under "Github"
    - Run `tarium.exe add Solarint/SAIN --no-checks` to add it to your profile
    - the `--no-checks` is needed because of the ratelimits, it will skip checking if it's a valid repo, so make sure you got the right one
    - if you want to be sure, just run it without the `--no-checks`
    - If you want to add multiple mods at once, just add more identifiers after the command like so:
        - `tarium.exe add Solarint/SAIN DrakiaXYZ/SPT-Waypoints DrakiaXYZ/SPT-BigBrain --no-checks`
    - until we get proper modpacks you can also create a text file with a list of mods you want to add, one per line, and use the `--file` option like so:
        - `tarium.exe add --file mods.txt --no-checks`

4. After adding all the mods you want, i recommend running `tarium.exe list` to see what you added
    - then run `tarium.exe download` to download and install them
    - you can also disable downloading, if you already ran it before (downloading also goes to the rate limit) like so:
        - `tarium.exe upgrade --no-download`

5. Finally: start SPT and enjoy the mods :)

### How to update:

To update the mods just run `tarium.exe download` or `tarium.exe upgrade` (might be easier to remember).

To update tarium itself, just download the new version from the releases page and replace the old tarium.exe with the new one.
I'll create an update-self command later, but for now this is the only way.

### Additional info:

There are alot of aliases for the commands too, you can see them all by running `tarium.exe --help` or `tarium.exe <command> --help` for a specific command.

The config file is located at "C:\Users\USER\AppData\Roaming\tarium\config\config.json" where USER is your windows username.
It contains all the profiles with their mod lists, but also the SPT folder you chose. Keep that in mind when you move your SPT folder somewhere else! (although it doesn't break it, it just won't work)

I added extensive logging to tarium, mostly for development but it also shows vital debugging information.
If you run into any problems, you can re-run a command with the `-v` flag to get more information about what is going on.
The more `-v` flags you add, the more verbose the output will be, here's the list:
- 0 => LevelFilter::Error
- 1 => LevelFilter::Warn
- 2 => LevelFilter::Info
- 3 => LevelFilter::Debug
- 4+ => LevelFilter::Trace

Keep in mind that the github connection also logs at debug level, which will cluter the output.
I recommend using `-v` for normal use, and `-vvv` if you run into any problems.

This will create a tarium.log file btw, located in the same folder as the tarium.exe file.
You can just ignore it normally, but if you run into any problems, you can send it to me and i'll try to help you out.

---

### Todo list:

# High Priority

- [x] remove old code that moves all files to .old
- [x] fix jar check in upgrade?!

- [ ] add logging for when filters couldnt match anything, or generally more logging...
    - [x] take custom logging i made from needs
    - [x] add more logging when filtering when adding
    - [x] add logging for all file io ops...

- [x] fix 7z? download fails for TommySoucy/MoreCheckmarks
    - [x] Add 7z extraction (e.g. sevenz-rust)
    - [ ] add support for dll file assets

- [x] fix mods not being deletable by user with admin right in explorer???!!
    - was just a readonly thing of .part files?
    - _sometimes_ i cannot delete folders with files/folders in them, i need to go into the most bottom folders and delete the files there, then go back up one by one to delete the mods folders

- [x] add option for upgrade subcommand to not download mods again

- [x] fix folder collapsing
    - [x] i think it should only collapse if there is folders nested with the same name
    - [x] i think it should only collapse when the archive has a single folder in it with the same name as the archive

- [ ] dont move archives to SPT folder for extraction, keep that in another folder, consider the root SPT(output_dir) folder as vital, aswell as the spt folder inside Bepinex/plugins

# General

- [ ] split up download and install commands from upgrade, and make upgrade just call them both
    - [ ] also rename upgrade to update

- [ ] make autocompletions work for clap

- [ ] add an option for the add subcommand to accept a file that has a list of mods to add
    - doesnt work because the indentifiers vec wants at least one mod
    - [ ] need another subcommand?

- [ ] make the profile switch command remove the old mods from the old profile
    - [ ] or at least warn the user that they need to do that manually

- [ ] add checks before downloading/extracting/moving files to see if there is enough space on the drive

- [ ] add checks before running any command  to see if the output_dir is a valid SPT installation
    - [ ] check for the right exes in the root

- [x] fix some mods
    - [x] Ram Cleaner Fix 1.2.2 - CactusPie/SPT-RamCleanerInterval
        - no zip, just a dll file
    - [x] space-commits/SPT-FOV-Fix
        - dll file in a zip that needs to be moved to bepinex/plugins
        - works with ui fixes tho, hmm...
        - where do the files from extract_temp get moved in the code??? log message for that is "installing extracted contents"

- [x] fix fucking github lol
    - the problem is that the graphql endpoint requires auth request, unlike the rest api which has the 60/h free...x
    - [ ] dedupe the get gh releases+assets thingy

- [x] change mod dir
    - [x] support for checking two dirs

- [ ] replace inquire with another one that doesnt suck on windows
    - [ ] try requestty - input duplication bug???
    - [ ] or askr (newer and not so sophisticated)

- [x] remove scan command as its not gonna work with dragged-in folders...

- [x] move processed archives to output_dir/MODS after successful extraction to reduce clutter.
- [x] when running upgrade with archives already downloaded it only downloads the files again, but it doesnt extract and move them into the folders as it should

- [ ] after extracting and moving mods, the ones in user/mods should get the package.json data checked for addition check for game version compatibility

- [ ] an option to force override mods, full redownload, full replace in mods folders

- [ ] handle rate limit error, also stop at the first one
    - [ ] add a rate limit request when fetching fails with that error message to get reset time stamp

- [ ] fix game versions filter bullshit somehow
    - [x] somewhat fixed, selection algo is better for sure
    - [ ] proper rework to get rid of all the filters eventually and just care about versions with a proper impl
    - [ ] more checks for spt version, description of release, maybe even scrape hub page (unlikely)
    - [ ] add mode where releases without a version are not installed until confirmed by the user, also show a link that opens a google search with that mod already entered

- [ ] maintain tests if possible...

# Future

- [ ] add a black list for shit like SVM who needs another app to setup fully?!

- [ ] add checks for incompatible fields in package.json

- [ ] hook up the hub as api?

- [ ] enable/disable mods
    - [ ] like curseforge maybe
    - [ ] look at archive file and match files to delete them from mods folders for disabling - basically "installing/uninstalling" them

- [ ] switch from cli to egui hehe
