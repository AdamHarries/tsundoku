use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result};

pub struct Entry {
    pub link: String,
    pub comment: String,
    pub tags: Vec<String>,
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

    /// assert_eq!(tag_id[0], 4);
    /// ```
    ///
    /// # Get a tag that doesn't exist
    /// ```
    /// # use tsundoku::datamodel::{Database, Entry};
    /// let db = Database::open_in_memory().unwrap();
    /// let tag_id = db.get_tag_id("This tag doesn't exist");
    /// assert_eq!(tag_id, Ok(vec![]));
    /// ```
    pub fn get_tag_id(&self, tag: &str) -> Result<Vec<u8>> {
        let mut stmt = self
            .conn
            .prepare("select tag_id from tags where tag == ?1")?;
        let tag_iter = stmt.query_map(params![tag], |row| Ok(row.get(0)?))?;
        tag_iter.collect()
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
        self.get_tag_id(tag).map(|v| v.len() > 0)
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
        self.conn.execute(
            "
            insert into links (link_id, link, comment)
                values (null, ?1, ?2)
        ",
            params![entry.link, entry.comment],
        )
    }

    pub fn add_tag(&self, tag: &str) -> Result<usize> {
        self.conn.execute(
            "insert into tags (tag_id, tag) values (NULL, ?1)",
            params![tag],
        )
    }
}

// #[cfg(test)]
// mod tests {
//     use super::{Database, Entry};

//     // Helper methods
//     fn add_tags_to_db(db: &Database) -> Vec<&str> {
//         let tags: Vec<&str> = vec!["tag 0", "tag 1", "tag 2", "tag 3", "tag 4"];

//         for tag in &tags {
//             db.add_tag(tag).unwrap();
//         }

//         tags
//     }

//     #[test]
//     fn get_existing_tag_id() {
//         let db = Database::open_in_memory().unwrap();
//         let _ = add_tags_to_db(&db);
//         let tag_id = db.get_tag_id("tag 3").unwrap();

//         assert_eq!(tag_id[0], 4);
//     }

//     #[test]
//     fn get_non_existing_tag_id() {
//         let db = Database::open_in_memory().unwrap();
//         let _ = add_tags_to_db(&db);
//         let tag_id = db.get_tag_id("This tag doesn't exist");

//         assert_eq!(tag_id, Ok(vec![]));
//     }
// }
