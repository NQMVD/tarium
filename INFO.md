Alright so... here's a mod manager i guess.

Disclaimer: it's a simple one, as of right now you still have to go to the mods page on the hub to get the ID.
but after you added them to your list, it's all just one command to download and install them. No need to extract, drag and drop them.

I'm still working on it, but in testing it works just fine with alot of mods for 3.11
The main limitation right now is that it only works with Github (the place where devs actually upload the mods) which means:
- no connection to the SPT hub -> no access to the description they have, only the github readme one which is mostly not even present
- because of no proper conventions sometimes no way to tell the SPT version the mod is for...
- HEAVY ratelimits for now, as i haven't registered the App on Github yet, we're talking 60 requests/h (i'll include some tips for that later, also this will change with both the registration and further improvements of the app)

This is a fork btw, i took the [ferium](https://github.com/gorilla-devs/ferium) tool as a base and swapped/ripped out the parts related to minecraft for their SPT counterparts. Mostly for the profile and github code it had, but also because i like terminal tools and LLMs work really well with Rust).
For now its just the terminal tool, but i already have plans to create a Wrapper around it, to give you guys a proper App you can click and drag around in. It will use a framework called eframe which in other cases showed crazy minimal resource usage, to not impact game performance.
That will have to wait tho, until the program is more polished and tested.

### How to install it:
> I **highly** recommend creating a copy of your SPT installation before you use tarium right now.
> Not because the program isn't safe, but it _is_ considered experimental and in such cases it's always advised to make a backup.

- Just download both files (tarium.exe and OPEN-CMD.bat) from [this](https://github.com/NQMVD/tarium/releases) page under "Assets" and place them both in your SPT folder, next to the SPT Launcher and Server exes.
- That's it.

> But if you know what you're doing, you can install it where-ever-you-want and run it from where-ever-you-want, that's why profiles have the SPT dir stored.

### how to use it:
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

To update the mods just run `tarium.exe download` or `tarium.exe upgrade` (might be easier to remember).

> There are alot of aliases for the commands too, you can see them all by running `tarium.exe --help` or `tarium.exe <command> --help` for a specific command.

> The config file is located at "C:\Users\USER\AppData\Roaming\tarium\config\config.json".
> It contains all the profiles with their mod lists, but also the SPT folder you chose. Keep that in mind when you move your SPT folder somewhere else! (although it doesn't break it, it just won't work)
