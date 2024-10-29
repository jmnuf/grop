# Grop

## The Poor Mans Grep
This is just a basic recreational remaking of some of grep's capabilities.
Made mostly because I use windows and I like grep but I'm not changing to linux just for some utilities that I can literally build myself for some everyday life file searching.

## Functionality List
I'm not making it do all the things grep can do, cause that's insane for a side-project.
So here's a list of what it can do and what I may make it do:
 - [x] Search current directory's files for a specific query string
 - [x] Query can be case insensitive (`-i` flag)
 - [x] Disable coloring if it bothers you for something (`-0` flag)
 - [x] Recursively go through the directory's subdirectories (`-r` flag)
 - [ ] Ability to pass in directories as arguments for where to search (TODO)
 - [ ] Be able to use simple regex like original grep (TODO)
 - [ ] Only display matched string without context (TODO)

## Build

I use [kinoko](https://github.com/jmnuf/kinoko) as my quick build system, instead of cargo just cause cargo does too much for such a simple project in my opinion. Kinoko is literally meant to be a thin wrapper over `rustc` and `rustc` is practically all you need to build this project.
Though you don't need to worry about using kinoko really, you can use the provided shell script to build grop, as long as `rustc` is found in your `%PATH%` (`$PATH` in linux).
If you are a firm believer of cargo, you can locally create your Cargo.toml but I assure you no Cargo.toml is ever entering this repo officially unless this project grows out of scope.
```bash
// linux
$ ./build.sh
// windows
$ .\build
```
The script should should compile grop onto the `build/` directory and execute the help command of grop.
```
 build/
   |_ grop[.exe]
   |_ grop.pdb
```
You can try out the following command to hopefully see that it kinda works
```bash
// linux
$ ./build/grop -r main
// windows
$ .\build\grop -r main
```

