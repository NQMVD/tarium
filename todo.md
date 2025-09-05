# high priority

- [ ] add logging for when filters couldnt match anything, or generally more logging...
    - [x] take custom logging i made from needs

- [x] fix 7z? download fails for TommySoucy/MoreCheckmarks
    - [x] Add 7z extraction (e.g. sevenz-rust)

- [x] fix mods not being deletable by user with admin right in explorer???!!
    - was just a readonly thing of .part files?


# current
- [x] fix fucking github lol
    - the problem is that the graphql endpoint requires auth request, unlike the rest api which has the 60/h free...x
    - [ ] dedupe the get gh releases+assets thingy
    - [ ] add a rate limit request when fetching fails with that error message

- [x] change mod dir
    - [x] support for checking two dirs

- [ ] replace inquire with another one that doesnt suck on windows
    - [ ] try requestty - looks nice too
        - [ ] try its progressbars, if they suck too, remove em
    - [ ] or askr (newer and not so sophisticated)

- [x] remove scan command as its not gonna work with dragged-in folders...

- [ ] use SPT\user\cache for downloads?
- [ ] move processed archives to output_dir/.old after successful extraction to reduce clutter.

- [ ] fix game versions filter bullshit somehow
    - [x] somewhat fixed, selection algo is better for sure

- [ ] after extracting and moving mods, the ones in user/mods should get the package.json data checked for addition check for game version compatibility

- [ ] downloaded zips should be in a subfolder called mod_zips

- [ ] check if theres an option to force override mods, full redownload, full replace in mods folders


# future

- [ ] hook up the hub as api?