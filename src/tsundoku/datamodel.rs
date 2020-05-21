use chrono::naive::NaiveDateTime;
use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result};

// TODO: Right now, the Tsundoku data model and database (i.e. sqlite)
// implementation are tied together. They should be separated out and the
// datamodel refactored into a trait with a separate database implementation.

//
// This is the rough overview of what the databse looks like.
//
// +-----------------------------------+
// | Links                             |     +---------------------------------+
// |                                   |     | LinkTags                        |
// | = id        : integer primary key <--+  |                                 |
// | = link      : text                |  |  | = id      : integer primary key |
// | = comments  : text                |  +--+ = link_id : integer foreign key |
// | = archive   : integer             |  +--+ = tag_id  : integer foreign key |
// | = timestamp : text                |  |  |                                 |
// |                                   |  |  +---------------------------------+
// +-----------------------------------+  |
//                                        |
// +----------------------------------+   |
// | Tags                             |   |
// |                                  |   |
// | = id  : integer primary key      <---+
// | = tag : text                     |
// |                                  |
// +----------------------------------+
//
// Rows in the 'Tags' table correspond directly to &str variable,
// rows in the Links table correspond (roughly) to an "Entry", associated with
// a particular archive.

/// Archive - a marker of where we "are" in reading a link. Right now, this means it's either in the queue (waiting to be read), or in the Archive (it's been read). This may be expanded to further archives (e.g. "InProgress", "ReReadLater") hence why it is an enum not a bool.
pub enum Archive {
    Queue,
    Archive,
}

/// Tag - a sorting/grouping string that can be used to query for specific entries
pub struct Tag<'a> {
    detail: &'a str,
}

/// Comment - A comment on a link, similar to a tag but semanticaly differnt: links/tags are many:many, but links/comments are 1:1
pub struct Comment<'a> {
    detail: &'a str,
}

/// Entry - An entry in the database
pub struct Entry<'a> {
    pub link: &'a str,              // Contents of the link
    pub comment: Option<&'a str>,   // Comment (optional) on the link
    pub tags: Option<Vec<&'a str>>, // Tags (also optional) for categorising the link
    pub archive: Archive,           // Have we read this link? Do we want to put it somewhere?
    pub timestamp: NaiveDateTime,   // When did we add this link to the database
}

/// The database of links
pub struct Database {
    conn: Connection,
}

#[derive(PartialEq, Debug)]
pub enum TagAddResult {
    TagAlreadyExists,
    TagId(i64),
}

#[derive(PartialEq, Debug)]
pub enum TagQueryResult {
    TagNotFound,
    TagId(i64),
}

impl PartialEq<TagQueryResult> for TagAddResult {
    fn eq(&self, other: &TagQueryResult) -> bool {
        match (self, other) {
            (TagAddResult::TagId(i), TagQueryResult::TagId(j)) => i == j,
            _ => false,
        }
    }
}
impl PartialEq<TagAddResult> for TagQueryResult {
    fn eq(&self, other: &TagAddResult) -> bool {
        match (self, other) {
            (TagQueryResult::TagId(i), TagAddResult::TagId(j)) => i == j,
            _ => false,
        }
    }
}

impl Database {
    pub fn open_in_memory() -> Result<Database> {
        let conn = Connection::open_in_memory()?;

        let rows = Database::init_database(&conn)?;

        return Ok(Database { conn: conn });
    }

    fn init_database(conn: &Connection) -> Result<usize> {
        // Create tables that we need, if they don't already exist.
        let mut rows = conn.execute(
            "
            create table if not exists links (
                link_id              INTEGER PRIMARY KEY AUTOINCREMENT,
                link            TEXT NOT NULL,
                comment         TEXT
            )",
            params![],
        )?;
        rows += conn.execute(
            "
            create table if not exists tags (
                tag_id              INTEGER PRIMARY KEY AUTOINCREMENT,
                tag             TEXT NOT NULL
            )",
            params![],
        )?;
        rows += conn.execute(
            "
            create table if not exists linktags (
                link_id         INTEGER,
                tag_id          INTEGER,
                FOREIGN KEY(link_id) REFERENCES links(link_id)
                FOREIGN KEY(tag_id) REFERENCES tags(tag_id)
            )",
            params![],
        )?;
        Ok(rows)
    }

    /// # Get a tag that exists
    ///
    /// ```
    /// # use tsundoku::datamodel::*;
    /// let db = Database::open_in_memory().unwrap();
    /// db.add_tag("tag 0");
    /// db.add_tag("tag 1");
    /// db.add_tag("tag 2");
    /// db.add_tag("tag 3");
    /// db.add_tag("tag 4");
    ///
    /// let tag_id = db.get_tag_id("tag 3").unwrap();
    ///
    /// assert_eq!(tag_id, TagQueryResult::TagId(4));
    /// ```
    ///
    /// # Get a tag that doesn't exist
    /// ```
    /// # use tsundoku::datamodel::*;
    /// let db = Database::open_in_memory().unwrap();
    /// let tag_id = db.get_tag_id("This tag doesn't exist");
    /// assert_eq!(tag_id, Ok(TagQueryResult::TagNotFound));
    /// ```
    pub fn get_tag_id(&self, tag: &str) -> Result<TagQueryResult> {
        let mut stmt = self
            .conn
            .prepare("select tag_id from tags where tag == ?1")?;
        let mut tag_iter = stmt.query_map(params![tag], |row| Ok(row.get(0)?))?;
        match tag_iter.next() {
            Some(Ok(i)) => Ok(TagQueryResult::TagId(i)),
            Some(Err(e)) => Err(e),
            None => Ok(TagQueryResult::TagNotFound),
        }
    }

    /// # Test if a database contains a tag
    /// ```
    /// use tsundoku::datamodel::*;
    /// let db = Database::open_in_memory().unwrap();
    /// db.add_tag("tag 0").unwrap();
    /// let tag_exists = db.contains_tag("tag 0").unwrap();
    /// assert!(tag_exists);
    /// ```
    pub fn contains_tag(&self, tag: &str) -> Result<bool> {
        match self.get_tag_id(tag)? {
            TagQueryResult::TagId(_) => Ok(true),
            TagQueryResult::TagNotFound => Ok(false),
        }
    }

    /// # List tags
    /// ```
    /// # use tsundoku::datamodel::*;
    /// let db = Database::open_in_memory().unwrap();
    /// let tags: Vec<&str> = vec!["tag 0", "tag 1", "tag 2", "tag 3", "tag 4"];
    /// for tag in &tags {
    ///     db.add_tag(tag);
    /// }
    /// let db_tags = db.list_tags().unwrap();
    /// assert_eq!(tags, db_tags);
    /// ```
    pub fn list_tags(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("select tag from tags;")?;
        let tag_iter = stmt.query_map(params![], |row| Ok(row.get(0)?))?;
        tag_iter.collect()
    }

    pub fn add_entry(&self, entry: Entry) -> Result<usize> {
        // set up values for the parameters
        let link = entry.link;
        let comment = match entry.comment {
            Some(c) => c,
            None => "",
        };
        let archive = Archive::Queue as u8; // we *always* add to the queue first
        let timestamp = entry.timestamp; // convert the timestamp to seconds

        // Add the link itself to the link table
        self.conn.execute(
            "
            insert into links (link_id, link, comment, archive, timestamp)
                values (null, ?1, ?2, ?3, ?4)
        ",
            params![link, comment, archive, timestamp],
        );

        // Get the ID of the entry we just pushed
        let link_id = self.conn.last_insert_rowid();

        // And iterate through the tags, pushing them to the db.
        match entry.tags {
            Some(ts) => {
                for tag in ts {
                    self.add_tag(tag);
                    // also need to link them!
                }
            }
            None => {} //nothing to do
        };

        Ok(0)
    }

    /// Add a tag to the database. If the tag already exists, this method does nothing.
    pub fn add_tag(&self, tag: &str) -> Result<TagAddResult> {
        self.contains_tag(tag).and_then(|contains| {
            if contains {
                Ok(TagAddResult::TagAlreadyExists)
            } else {
                self.conn
                    .execute(
                        "insert into tags (tag_id, tag) values (NULL, ?1)",
                        params![tag],
                    )
                    .map(|_| TagAddResult::TagId(self.conn.last_insert_rowid()))
            }
        })
    }

    fn tag_link(&self, tag_id: i64, link_id: i64) {}
}

#[cfg(test)]
mod test {
    use super::*;
    // Tests for private members.
    #[test]
    fn add_tag_get_tag_same() {
        let db = Database::open_in_memory().unwrap();
        // Add some tags so that we don't just have zero
        let tags: Vec<&str> = vec!["tag 0", "tag 1", "tag 2", "tag 3", "tag 4"];

        for tag in &tags {
            db.add_tag(tag).unwrap();
        }
        // Add a tag and get the id from that
        let add_id = match db.add_tag("test tag").unwrap() {
            TagAddResult::TagAlreadyExists => panic!("tag should not already exist!"),
            TagAddResult::TagId(i) => i,
        };
        // Query for the tag, and get the id
        let query_id = match db.get_tag_id("test tag").unwrap() {
            TagQueryResult::TagNotFound => panic!("Tag should exist and show up in query!)"),
            TagQueryResult::TagId(i) => i,
        };

        assert_eq!(add_id, query_id);
    }
}
