
- [x] fix fucking github lol
    - the problem is that the graphql endpoint requires auth request, unlike the rest api which has the 60/h free...x
    - [ ] dedupe the get gh releases+assets thingy
    - [ ] add a rate limit request when fetching fails with that error message

- [x] change mod dir
    - [x] support for checking two dirs

- [ ] replace inquire with another one that doesnt suck on windows
    - [ ] try requestty
        - [ ] try its progressbars, if they suck too, remove em
    - [ ] or askr (newer and not so sophisticated)

- [ ] remove scan command as its not gonna work with dragged-in folders...

- [ ] use SPT\user\cache for downloads?

- [ ] hook up the hub as api?

- [ ] fix game versions filter bullshit somehow

- [ ] after extracting and moving mods, the ones in user/mods should get the package.json data checked for addition check for game version compatibility

- [ ] downloaded zips should be in a subfolder called mod_zips

- [ ] add logging for when filters couldnt match anything, or generally more logging...

- [ ] fix mods not being deletable by user with admin right in explorer???!!