# marl

Simple Deezer ARL manager. Useful for `streamrip`, `deemix`, etc.
Depends on access to [Firehawk's list](https://rentry.co/firehawk52).

```bash
# Gets the first valid ARL from the top of the stack
$ marl
6b2c2bf[...]

# You can also get ARLs from a specific region
$ marl -r Brazil
4b4ees0[...]

# marl can also edit your config files for you, if the ARL doesn't work anymore
$ marl invalidate
$ marl config streamrip
```

## To-do
- [x] Cache ARLs to disk
- [x] Select ARL by region
- [x] Invalidate last used ARL (per region) if not working
- [ ] Options to update config files for ripper programs (in progress)
