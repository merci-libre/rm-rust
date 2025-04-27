# rm-rust
Part of the Merci-Libre coreutils collection. Rust rewrite of the 'rm' command.

### missing features:
`--no-preserve-root` : program will not delete the root directory or any file with 000 permissions. Even with SUID bit set, this command will not run on the root directory.
This will be fixed in the coming weeks.
