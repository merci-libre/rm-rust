mod args;
use args::*;
use clap::Parser;
use std::env::current_dir;
use std::fs::{exists, read_dir, remove_dir, remove_dir_all, remove_file};
use std::io::{stdin, ErrorKind};
use std::path::PathBuf;
use std::process::exit;
use std::usize;

fn main() {
    let args = Removeargs::parse();
    let file_count = args.file.len();
    if file_count == 0 {
        eprintln!("[Error] no files were inputted. Try running with '-h' or '--help' for more information.");
        exit(1);
    }
    let mut mode: u8 = 9;
    let mut continue_after_root: bool = false;

    /*interactive options*/
    // match arg -f
    match args.force {
        true => mode = 0,
        false => (),
    }
    // match arg -I
    match args.x {
        true => mode = 1,
        false => (),
    }
    // match arg -i
    match args.i {
        true => mode = 2,
        false => (),
    }
    if mode == 9 {
        // no options were given...
        match args.interactive.to_lowercase().as_str() {
            "never" => mode = 0, // remove files and directory
            "once" => mode = 1, // check before removing files and directory if greater than 3 files
            // deleted
            "always" => mode = 2, // always check before removing files
            _ => (),
        }
    }
    /*preserve root options*/

    // match --no-preserve-root
    match args.no_preserve_root {
        true => mode = 5,
        false => (),
    }
    //match '--preserve-root'
    match args.preserve_root.to_lowercase().as_str() {
        "none" => continue_after_root = true,
        "all" => continue_after_root = false,
        _ => {
            eprintln!("[Error] {} is not a valid option for --preserve-root, please see '-h' or '--help' for more information.",args.preserve_root);
            exit(1);
        }
    }
    if args.debug {
        dbg!(
            "ARG:FILES PROPERTIES",
            file_count,
            args.file.clone(),
            "REMOVAL ARGUMENTS",
            args.i,
            args.force,
            args.x,
            args.interactive,
            "PRESERVE ROOT ARGS",
            args.preserve_root,
            args.no_preserve_root,
            "LOOP PARAMETERS",
            &mode
        );
    }
    /*
     * main loop Starts here.
     */
    let mut count = 0;
    for i in args.file.clone() {
        let filename = i.clone();
        if args.verbose || args.debug {
            if mode == 6 {
                eprintln!(
                    "[WARNING] a soft error occurred when trying to delete {},",
                    args.file[count - 1]
                );
            }
            dbg!(mode);
            eprintln!("[Verbose] removing '{}'", i.clone());
        }
        mode = checkmode(
            filename,
            file_count,
            mode,
            args.recursive,
            args.dir,
            args.force,
            continue_after_root,
        );
        count += 1;
    }
}

fn user_prompt(path: PathBuf, filename: &String, mode: u8, len: usize, is_dir: bool) -> bool {
    let mut file = filename
        .split("/")
        .last()
        .map(str::trim)
        .unwrap()
        .to_string();
    let size = match len {
        0 => exit(1),
        1 => file.to_string(),
        1..100 => format!("{} files", len).to_string(),
        100 => ("100 files").to_string(),
        _ => ("more than 100 files").to_string(),
    };
    let directory = filename
        .split("/")
        .next()
        .map(str::trim)
        .unwrap()
        .to_string();

    if is_dir {
        file.push_str(" (directory)");
    } else {
        file.push_str(" (regular file)");
    }
    match mode {
        1 => {
            // prompt once
            eprint!(
                "\n[WARNING] you are about to delete {} in {}/{}.\nTo confirm this, press [y/n]: ",
                size,
                path.display(),
                directory,
            );
        }
        2 => {
            // prompt always
            eprint!(
                "\n[WARNING] you are about to delete {} in {}/{}.\nTo confirm this, press [y/n]: ",
                file,
                path.display(),
                directory,
            );
        }
        _ => todo!(),
    }

    //take stdin
    let mut yorn = String::new();
    yorn.clear();
    stdin().read_line(&mut yorn).unwrap();

    loop {
        match yorn.to_lowercase().trim() {
            "y" => return true,
            "n" => return false,
            _ => eprintln!("please enter either 'y' or 'n'."),
        }
    }
}

fn checkmode(
    filename: String,
    file_count: usize,
    mut mode: u8,
    recursive: bool,
    dir: bool,
    force: bool,
    continue_after_root: bool,
) -> u8 {
    /*
     * error parsing before throwing into match statement.
     *
     * This entire function checks the mode before deleting the file.
     * The bounds of the function are set when this function is called
     * and is defined through the variable "mode".
     *
     * Mode gets constantly redefined, and to keep track of what it does,
     * I have commented what each mode does.
     *
     * As a reference, you can also use this key here to keep track:
     *
     * 0-> force delete all files
     * 1-> prompt before deletion of files if the file count exceeds 3. Invoked by option -I,
     * 2-> prompt always for each file deleted.
     * 3-> delete directories recursively,
     * 4-> delete empty directories only.
     * 5-> same as 3, but ONLY invoked by deleting the root directory.
     * 6-> A soft error is thrown, but the program continues.
     *
     * You may have noticed at the start of the program, the mode is set to '9' by default. This is
     * because by default the mode should get changed after CLAP parses the argument and the "mode"
     * gets redefined by the structs within args.
     *
     * So essentially the way this program works is like such:
     *
     * main defines mode
     * |            |->>checkmode determines the mode depending on file input (is dir, file, or empty_dir)
     * |--<<------------return 'mode' to main loop<<-----------<-------------if all is OK, delete files<-|
     *
     *
     *
     */

    //catches the path, and sets it to a variable
    let path = match current_dir() {
        Ok(v) => v,
        Err(_e) => {
            eprintln!("An unknown error has occured.");
            exit(1)
        }
    };

    // exits if file does not exist.
    let _exists = match exists(filename.clone()) {
        Ok(false) => {
            eprintln!(
                "[Error] {}/{filename} is either missing, or does not exist.",
                path.display()
            );
            exit(1)
        }
        /*Some unknown error*/
        Err(_e) => {
            eprintln!("[Error] something went wrong when searching for the file, exiting program.");
            exit(1);
        }
        /*File does exist*/
        Ok(true) => (),
    };
    // check if a file is directory, first before opening file?
    let check_directory = std::fs::metadata(filename.clone()).unwrap();
    let is_dir = check_directory.is_dir();

    // stops deletion of root directory.
    if filename == ("/") && (mode == 0 || mode == 3) && !(mode == 5) {
        eprintln!("[WARNING] It is dangerous to run this recursively on the root directory. \nRun with --no-preserve-root to override this fail save.");
        dbg!(mode);
        if continue_after_root {
            mode = 6; // do nothing and continue.
        } else {
            exit(1);
        }
    }
    // deletes if mode is set to zero from mode 1.

    if recursive && (force || mode == 0) && is_dir {
        mode = 3
    }

    // if recursive and not force:
    if recursive && is_dir && mode != 3 {
        mode = 2;
    }

    //handles whether a directory should be deleted if it is empty, otherwise continue.
    if is_dir && dir && !force {
        let directory_empty = read_dir(filename.clone()).unwrap().next().is_none();
        if directory_empty {
            mode = 4 // delete only if empty
        } else {
            eprintln!("[Error] cannot remove '{}': Directory not empty", filename);
            mode = 6; //do nothing and continue.
        }
    }

    //stops deletion of directory if -r, -d is not specified.

    if is_dir && !(mode == 3 || mode == 4 || mode == 6) && recursive == false {
        eprintln!("[Error] cannot remove '{}': Is a directory", filename);
        mode = 6; // do nothing and continue
    }

    /*
     * Deletion happens here.
     */
    match mode {
        0 => {
            // force deletion.
            remove(filename, 0);
        }
        1 => {
            // if deleting more than 3 files, check before deletion.
            if user_prompt(path, &filename, mode, file_count, is_dir) {
                remove(filename, 0);
            } else {
                exit(1);
            }
            mode = 0;
        }
        2 => {
            // always check before removing files
            if user_prompt(path, &filename, mode, file_count, is_dir) {
                if is_dir {
                    remove(filename, 1);
                } else {
                    remove(filename, 0);
                }
            }
        }
        3 => {
            // delete a directory recursively.
            remove(filename, 1);
        }
        4 => {
            // delete only if empty directory
            remove(filename, 2);
        }
        5 => {
            // this will delete the root directory. It is the same as option 3, but I have decided
            // it is best to keep option 3 and this option separated as it is a very dangerous
            // command that deserves it's own mode of deletion.
            remove(filename, 1);
        }
        _ => (),
    }
    return mode;
}
fn remove(filename: String, types: u8) {
    /*
     * more error checking... added for readability.
     * */
    match types {
        0 => {
            // remove file
            match remove_file(&filename) {
                Ok(_v) => (),
                Err(ref e) if e.kind() == ErrorKind::PermissionDenied => {
                    eprintln!(
                        "[ERROR] Cannot remove '{}': Permission denied",
                        filename.clone()
                    );
                    exit(1);
                }
                Err(_e) => {
                    eprintln!("Something went wrong, exiting program");
                    dbg!(_e);
                    exit(1);
                }
            };
        }
        1 => {
            //recursive deletion (whole directory).
            match remove_dir_all(filename) {
                Ok(_v) => (),
                Err(ref e) if e.kind() == ErrorKind::PermissionDenied => {
                    eprintln!("[ERROR] You must be root to run this command.");
                    exit(1);
                }
                Err(_e) => {
                    eprintln!("Something went wrong, exiting program");
                    dbg!(_e);
                    exit(1);
                }
            };
        }
        2 => {
            // remove empty directory
            match remove_dir(&filename) {
                Ok(_v) => (),
                Err(ref e) if e.kind() == ErrorKind::PermissionDenied => {
                    eprintln!(
                        "[ERROR] Cannot remove '{}': Permission denied",
                        filename.clone()
                    );
                    exit(1);
                }
                Err(_e) => {
                    eprintln!("Something went wrong, exiting program");
                    dbg!(_e);
                    exit(1);
                }
            };
        }
        _ => todo!(),
    }
}
