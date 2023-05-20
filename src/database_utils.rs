use sqlite::{Error, Connection};
use std::collections::HashMap;

pub struct Database {
    filepath: String,
}

#[allow(dead_code)]
impl Database {
    pub fn new(filepath: &str) -> Database {
        Database {filepath: filepath.to_string()}
    }

    pub fn request_row(&self, table: &str, key_column: &str, key: &str) -> Result<HashMap<String, String>, Error> {
        let mut result_map: HashMap<String, String> = HashMap::new();
        let connection = Connection::open_with_full_mutex(&self.filepath)?;

        let sql_request = format!("SELECT * FROM {} WHERE {} = ?", table, key_column);
        let mut statement = connection.prepare(sql_request)?;
        statement.bind((1, key))?;
        if let sqlite::State::Done = statement.next()? {
            return Ok(result_map);
        };

        let column_count: usize = statement.column_count();
        for k in 0..column_count {
            let key = statement.column_name(k)?;
            let value = statement.read::<String, _>(k).unwrap_or_default();
            result_map.insert(key.to_string(), value.to_string());
        }
        Ok(result_map)
    }

    pub fn push_data(&self, table: &str, values: HashMap<String, String>) -> Result<(), Error> {
        let connection = Connection::open_with_full_mutex(&self.filepath)?;
        let mut sql_request = format!("INSERT INTO {} (", table);
        let mut sql_values = vec![];

        for (column_name, value) in values.iter() {
            sql_request.push_str(&format!("{}, ", column_name));
            sql_values.push(value);
        }

        // remove trailing comma and space
        sql_request.pop();
        sql_request.pop();

        sql_request.push_str(") VALUES (?");
        for _ in 1..sql_values.len() {
            sql_request.push_str(", ?");
        }
        sql_request.push(')');

        let mut statement = connection.prepare(&sql_request)?;

        for (i, value) in sql_values.iter().enumerate() {
            statement.bind((i + 1, value.as_str()))?;
        }

        statement.next()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::database_utils::*;
    #[test]
    fn test_request_row() {
        let database = Database::new("database.db");
        let row = database.request_row("users", "username", "admin");
        assert_eq!(HashMap::from([]), row)
    }

    #[test]
    fn test_push_data() {
        let mut data: HashMap<&str, &str> = HashMap::new();
        data.insert("username", "user1");
        data.insert("hash", "hash");
        data.insert("credits","150");
        data.insert("auth_level","1");
        data.insert("email","user1@example.com");
        data.insert("sessionID","1234");
        data.insert("sessionExpires","1");

        let values = data.iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let database = Database::new("database.db");
        database.push_data("users", values);
    }
}