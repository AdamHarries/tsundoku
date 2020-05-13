# DESIGN.md

Design discussion for Tsundoku.

## Workflows

### Example 1.

Adam wants to add a link to his dump, so he invokes `tsd`, the command line app. Later, when he wants to find something to read, he invokes `tsd` to show his dump, queries it for something he wants to read, then marks it as read.

Step one, he adds it to the dump:

    > tsd add "https://music.stackexchange.com/questions/99546/making-sense-of-blues-soloing-differentiating-major-minor-pentatonics" -c "Interesting stackoverflow question about major and minor while playing the blues" -t "music, blues, soloing, improvisation"

    Added to queue, id #deadbeef

This example includes the `-c` and `-t` flags, which allow adam to add a "comment" and some "tags" to the link, which can be used to sort and query the dump or archive.

Step two, he wants to find something to read:

    > tsd bored

    Tsundoku queue:
    - #deadbeef -- https://music.stackexchange.com/questions/99546/making-sense-of-blues-soloing-differentiating-major-minor-pentatonics
        =c= Interesting stackoverflow question about major and minor while playing the blues
        =t= music, blues, soloing, improvisation

    - #a34ebff5 -- https://docs.rs/clap/2.33.1/clap/struct.SubCommand.html
        =c= Docs for the clap library that tsunduko could use
        =t= docs, programming, rust

    ..

Step three, he decides to read the link he added earlier, so marks it as read, which adds it to his archive.

    > tsd read #deadbeef

    Link:
        https://music.stackexchange.com/questions/99546/making-sense-of-blues-soloing-differentiating-major-minor-pentatonics
    moved from dump to archive.

### Example 2.

Adam has read an interesting link, and wants to add it to the archive directly, but without tags or commentary (he's in a rush).

    > tsd add -a "https://music.stackexchange.com/questions/99546/making-sense-of-blues-soloing-differentiating-major-minor-pentatonics"

The flag `-a` stands for "archive", which will automatically send the link to the archive, rather than the dump (the normal behaviour)