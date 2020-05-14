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

    pub fn get_tag_id(&self, tag: String) -> Result<Vec<u8>> {
        let mut stmt = self
            .conn
            .prepare("select tag_id from tags where tag == ?1")?;
        let tag_iter = stmt.query_map(params![tag], |row| Ok(row.get(0)?))?;
        tag_iter.collect()
    }

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

    pub fn add_tag(&self, tag: String) -> Result<usize> {
        self.conn.execute(
            "insert into tags (tag_id, tag) values (NULL, ?1)",
            params![tag],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Database, Entry};
    #[test]
    fn list_tags() {
        let db = Database::open_in_memory().unwrap();

        let tags: Vec<String> = vec![
            String::from("tag 0"),
            String::from("tag 1"),
            String::from("tag 2"),
            String::from("tag 3"),
            String::from("tag 4"),
        ];

        for tag in &tags {
            db.add_tag(tag.clone()).unwrap();
        }

        let db_tags = db.list_tags().unwrap();

        assert_eq!(tags, db_tags);
    }

    #[test]
    fn get_existing_tag_id() {
        let db = Database::open_in_memory().unwrap();

        let tags: Vec<String> = vec![
            String::from("tag 0"),
            String::from("tag 1"),
            String::from("tag 2"),
            String::from("tag 3"),
            String::from("tag 4"),
        ];

        for tag in &tags {
            db.add_tag(tag.clone()).unwrap();
        }

        let tag_id = db.get_tag_id(String::from("tag 3")).unwrap();

        assert_eq!(tag_id[0], 4);
    }
}
