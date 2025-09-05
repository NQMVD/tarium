# high priority

- [ ] add logging for when filters couldnt match anything, or generally more logging...
    - [x] take custom logging i made from needs

- [x] fix 7z? download fails for TommySoucy/MoreCheckmarks
    - [x] Add 7z extraction (e.g. sevenz-rust)

- [x] fix mods not being deletable by user with admin right in explorer???!!
    - was just a readonly thing of .part files?
    - _sometimes_ i cannot delete folders with files/folders in them, i need to go into the most bottom folders and delete the files there, then go back up one by one to delete the mods folders


# current
- [x] fix fucking github lol
    - the problem is that the graphql endpoint requires auth request, unlike the rest api which has the 60/h free...x
    - [ ] dedupe the get gh releases+assets thingy

- [x] change mod dir
    - [x] support for checking two dirs

- [ ] replace inquire with another one that doesnt suck on windows
    - [ ] try requestty - looks nice too
        - [ ] try its progressbars, if they suck too, remove em
    - [ ] or askr (newer and not so sophisticated)

- [x] remove scan command as its not gonna work with dragged-in folders...

- [x] move processed archives to output_dir/MODS after successful extraction to reduce clutter.
- [x] when running upgrade with archives already downloaded it only downloads the files again, but it doesnt extract and move them into the folders as it should

- [ ] fix game versions filter bullshit somehow
    - [x] somewhat fixed, selection algo is better for sure
    - [ ] proper rework to get rid of all the filters eventually and just care about versions with a proper impl

- [ ] after extracting and moving mods, the ones in user/mods should get the package.json data checked for addition check for game version compatibility

- [ ] an option to force override mods, full redownload, full replace in mods folders

- [ ] handle rate limit error, also stop at the first one
    - [ ] add a rate limit request when fetching fails with that error message to get reset time stamp


# future

- [ ] hook up the hub as api?

- [ ] enable/disable mods
    - [ ] like curseforge maybe
    - [ ] look at archive file and match files to delete them from mods folders for disabling - basically "installing/uninstalling" them