use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result};

pub struct Entry<'a> {
    pub link: &'a str,
    pub comment: &'a str,
    pub tags: Vec<&'a str>,
}

//
// This is the rough overview of what the databse looks like.
//
// +----------------------------------+
// | Links                            |     +---------------------------------+
// |                                  |     | LinkTags                        |
// | = id       : integer primary key <--+  |                                 |
// | = link     : text                |  |  | = id      : integer primary key |
// | = comments : text                |  +--+ = link_id : integer foreign key |
// |                                  |  +--+ = tag_id  : integer foreign key |
// +----------------------------------+  |  |                                 |
//                                       |  +---------------------------------+
// +----------------------------------+  |
// | Tags                             |  |
// |                                  |  |
// | = id  : integer primary key      <--+
// | = tag : text                     |
// |                                  |
// +----------------------------------+

pub struct Database {
    conn: Connection,
}

pub enum TagAddResult {
    TagAlreadyExists,
    TagId(i64),
}

pub enum TagQueryResult {
    TagNotFound,
    TagId(i64),
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
    /// # use tsundoku::datamodel::{Database, Entry};
    /// let db = Database::open_in_memory().unwrap();
    /// db.add_tag("tag 0");
    /// db.add_tag("tag 1");
    /// db.add_tag("tag 2");
    /// db.add_tag("tag 3");
    /// db.add_tag("tag 4");
    ///
    /// let tag_id = db.get_tag_id("tag 3").unwrap();
    ///
    /// assert_eq!(tag_id[0], 4);
    /// ```
    ///
    /// # Get a tag that doesn't exist
    /// ```
    /// # use tsundoku::datamodel::{Database, Entry};
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
        // tag_iter.collect()[0];
    }

    /// # Test if a database contains a tag
    /// ```
    /// use tsundoku::datamodel::{Database, Entry};
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
    /// # use tsundoku::datamodel::{Database, Entry};
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
        self.add_link(entry.link, entry.comment)
    }

    // Add a link to the 'links' table
    fn add_link(&self, link: &str, comment: &str) -> Result<usize> {
        // Add the link itself to the link table
        self.conn.execute(
            "
            insert into links (link_id, link, comment)
                values (null, ?1, ?2)
        ",
            params![link, comment],
        )
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
        let query_id = db.get_tag_id("test tag").unwrap()[0];

        assert_eq!(add_id, query_id);
    }
}
