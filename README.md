# PkRoam
Let your monsters roam freely!

This is a collection of tools for editing generation 3 GameBoy Advance game save files. There is a `pktools` binary with various subcommands:
* `inspect` - Look through data for boxes and party Pokemon
* `extract` - Take a Pokemon from a save file (deleting it) and save the data to a file
* `insert` - Insert a Pokemon's data into a specific save slot after extracting it into a file (from previous command).

## Goals

The primary project for this repository is the `pkroam` application which intends to function and behave like the various Box/Bank/Home supplementary games for storing Pokemon data outside of the games themselves. I'm currently building out the backend with a CLI tool `pkroam-cli` which allows testing functionality as I build it.

```sh
$ target/debug/pkroam-cli
Usage: pkroam-cli [OPTIONS] <COMMAND>

Commands:
  deposit     
  list-saves  
  list-mons   
  help        Print this message or the help of the given subcommand(s)

Options:
  -c, --config-dir <CONFIG_DIR>  
      --enable-debug             
  -h, --help                     Print help
```

Eventually, this project will also include a graphic tool which utilizes `pkroam-backend` in order to administer Pokemon binary data in a user-friendly manner.

## Differences with PKHex

PKHex is a great tool! It allows modifying Pokemon save files and the Pokemon inside arbitrarily. You can boost a Pokemon to level 100, give it moves it doesn't normally have, or just give yourself infinite Master Balls. The tool primarily functions as a one-stop shop for save editing.

In contrast, this tool intends to give a "vanilla" save editing experience. Pokemon can be taken from a save file and stored, then later inserted into a different save file; perhaps of a different generation. For players who primarily use ROMs, this should give an experience more similar to console players and can be helpful for organization or tracking your collection.

## Roadmap

That's all well and good, but it's a relatively large project. Right now I've managed to get initial storage of generation 3 Pokemon data implemented, but no further. I intend to get withdrawals added and then start working on generation 2 data. Following that, I want to come up with a conversion method similar to the way Pokemon Home handles data from generations 8 and 9, allowing backwards conversion with some resets to things like EVs, movesets, etc. Once those processes are figured out in a way that makes sense, is tested, and seems easy to maintain, I'll start adding support for more data formats. At some point along the line, there will be a need for a GUI, but I'm not hungry for that just yet.