extern crate tsundoku;

use tsundoku::datamodel::{Database, Entry};

#[macro_use]
extern crate clap;
use clap::App;

fn main() {
    let matches = clap_app!(myapp =>
        (version: "0.0.1") // Use semver https://semver.org/
        (author: "Adam H <harries.adam@gmail.com>")
        (about: "blesh")
        (@arg CONFIG: -c --config +takes_value "Sets a custom config file")
        (@arg debug: -d ... "Sets the level of debugging information")
        (@subcommand add =>
            (about: "Add a link or reference to a piece to the pile.")
            (version: "0.0.1") // use semver
            (@arg LINK: +required "The link or reference to add to the pile.")
            (@arg COMMENT: -c --comment +takes_value "A comment on the link for later reference")
            (@arg TAGS: -t --tags +takes_value "A comma separated list of tags to associate with the link")
        )
        (@subcommand read =>
            (about: "Pull a link from the dump, mark it as read, and add it to the archive.")
            (version: "0.0.1") //use semver
            (@arg ID: +required "The ID of the link to read")
        )
        // (@subcommand bored  =>
        //     (about: "Find something to read, aka dump the list of things to read.")
        //     (version "0.0.1")
        // )
    )
    .get_matches();

    println!("Got matches: {:?}", matches);

    println!("Hello, world!");
}
