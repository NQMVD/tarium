# high priority

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

# current

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


# future

- [ ] add a black list for shit like SVM who needs another app to setup fully?!

- [ ] add checks for incompatible fields in package.json

- [ ] hook up the hub as api?

- [ ] enable/disable mods
    - [ ] like curseforge maybe
    - [ ] look at archive file and match files to delete them from mods folders for disabling - basically "installing/uninstalling" them
