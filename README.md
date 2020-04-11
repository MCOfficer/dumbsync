# dumbsync

A dumb rsync-like for fetching/updating a folder from a server.

This repository contains both the dumbsync library (`dumbsync`) as well as `dumbsync-cli`, an ultra-thin CLI wrapper around it.
Be advised that this is by no means stable, probably terrible code, and i certainly wouldn't use it in production.

## How does it work?

The remote directory is expected to contain a top-level file called `.dumbsync`, which contains all subfile-paths and their respective BLAKE3-hashes:
```
dbb4f489f9d3d6a35190d68092cff05e51673c8068691c61c7d637b41b9f3593 data/coalition/coalition outfits.txt
9d6874c149de2cf7d79fd977bfe55f4b9ecbfb952a05a8a7f3c28ff9d2ac679a data/coalition/coalition.txt
6ef795abeedf24435d11a932c9731869991694688b48f51f6adb9957de0711bf keys.txt
```

The client downloads this file, and checks whether the files already exist, and if they do, hashes them to see if they're uptodate.
Based on this, it downloads all files that need updates and optionally purges all local files that do not exist on the remote.

The hashing happens on a per-file basis, there is no fancy chunking going on, so it will redownload any huge files that have been changed. **dumbsync only makes sense for tons of small files.**

## dumbsync-cli

One can generate a `.dumbsync` file for a given directory like so:
```bash
$ dumbsync-cli generate /path/to/directory
```
This will recurse the entire directory and create a `.dumbsync` file there.

Downloading is equally simple: 
```bash
$ dumbsync-cli download https://example.com/path/to/directory /directory/to/sync/to/ [--purge]
```

## Contributing

Please don't. No, seriously, there are tons of better projects out there. I just wrote this because it perfectly fits my needs, and i needed to get my feet wet. If you want to help or don't trust my coding skills enough (surprise :) ), check these out:
- [bita](https://github.com/oll3/bita): Very promising and my personal favourite, but [needs help](https://github.com/oll3/bita/issues/13) to be usable as library.
- [fast_rsync](https://github.com/dropbox/fast_rsync): rather low-level rsync implementation with room for improvements.
- [lms](https://github.com/wchang22/lumins): Good in its own niche, but only CLI and not for remote files.
- [librsync](https://crates.io/crates/librsync) and [librsync-ffi](https://crates.io/crates/librsync-ffi): Probably the most rsync-esque you'll get, if you're comfortable binding to a C library.
- [rusync](https://github.com/dmerejkowsky/rusync): Again, only CLI and only local.
- [rustsync](https://nest.pijul.com/pmeunier/rustsync): Seems dead, unfortunately.