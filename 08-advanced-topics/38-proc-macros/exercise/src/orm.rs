//! The runtime half of Exercise 2: SQL values, column metadata and the
//! `Table` trait (provided). `#[derive(Table)]` implements `TABLE`,
//! `columns`, `values` and `from_row`; the SQL statements are built here
//! from those.

use std::fmt;

/// A value in a SQL row (SQLite's storage classes).
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrmError {
    WrongColumnCount {
        expected: usize,
        got: usize,
    },
    TypeMismatch {
        column: String,
        expected: &'static str,
    },
}

impl OrmError {
    /// Attach the column name to a type error.
    pub fn in_column(self, column: &str) -> Self {
        match self {
            OrmError::TypeMismatch { expected, .. } => OrmError::TypeMismatch {
                column: column.to_string(),
                expected,
            },
            other => other,
        }
    }
}

impl fmt::Display for OrmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrmError::WrongColumnCount { expected, got } => {
                write!(f, "expected {expected} columns, got {got}")
            }
            OrmError::TypeMismatch { column, expected } => {
                write!(f, "column {column}: expected {expected}")
            }
        }
    }
}

impl std::error::Error for OrmError {}

/// A Rust type that maps to a SQL column type.
pub trait SqlType: Sized {
    const SQL: &'static str;
    const NULLABLE: bool = false;
    fn to_value(&self) -> Value;
    fn from_value(value: &Value) -> Result<Self, OrmError>;
}

fn mismatch(expected: &'static str) -> OrmError {
    OrmError::TypeMismatch {
        column: String::new(),
        expected,
    }
}

impl SqlType for i64 {
    const SQL: &'static str = "INTEGER";
    fn to_value(&self) -> Value {
        Value::Integer(*self)
    }
    fn from_value(value: &Value) -> Result<Self, OrmError> {
        match value {
            Value::Integer(i) => Ok(*i),
            _ => Err(mismatch("INTEGER")),
        }
    }
}

impl SqlType for i32 {
    const SQL: &'static str = "INTEGER";
    fn to_value(&self) -> Value {
        Value::Integer(*self as i64)
    }
    fn from_value(value: &Value) -> Result<Self, OrmError> {
        match value {
            Value::Integer(i) => i32::try_from(*i).map_err(|_| mismatch("INTEGER (32-bit)")),
            _ => Err(mismatch("INTEGER")),
        }
    }
}

impl SqlType for bool {
    /// SQLite has no boolean: 0 or 1.
    const SQL: &'static str = "INTEGER";
    fn to_value(&self) -> Value {
        Value::Integer(*self as i64)
    }
    fn from_value(value: &Value) -> Result<Self, OrmError> {
        match value {
            Value::Integer(0) => Ok(false),
            Value::Integer(1) => Ok(true),
            _ => Err(mismatch("INTEGER 0 or 1")),
        }
    }
}

impl SqlType for f64 {
    const SQL: &'static str = "REAL";
    fn to_value(&self) -> Value {
        Value::Real(*self)
    }
    fn from_value(value: &Value) -> Result<Self, OrmError> {
        match value {
            Value::Real(r) => Ok(*r),
            Value::Integer(i) => Ok(*i as f64),
            _ => Err(mismatch("REAL")),
        }
    }
}

impl SqlType for String {
    const SQL: &'static str = "TEXT";
    fn to_value(&self) -> Value {
        Value::Text(self.clone())
    }
    fn from_value(value: &Value) -> Result<Self, OrmError> {
        match value {
            Value::Text(s) => Ok(s.clone()),
            _ => Err(mismatch("TEXT")),
        }
    }
}

impl<T: SqlType> SqlType for Option<T> {
    const SQL: &'static str = T::SQL;
    const NULLABLE: bool = true;
    fn to_value(&self) -> Value {
        self.as_ref().map_or(Value::Null, T::to_value)
    }
    fn from_value(value: &Value) -> Result<Self, OrmError> {
        match value {
            Value::Null => Ok(None),
            v => T::from_value(v).map(Some),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Column {
    pub name: &'static str,
    pub sql_type: &'static str,
    pub nullable: bool,
    pub primary_key: bool,
}

pub trait Table: Sized {
    const TABLE: &'static str;
    fn columns() -> &'static [Column];
    /// This row's values, in column order.
    fn values(&self) -> Vec<Value>;
    fn from_row(row: &[Value]) -> Result<Self, OrmError>;

    /// `CREATE TABLE people (id INTEGER PRIMARY KEY, name TEXT NOT NULL, email TEXT)`
    fn create_table_sql() -> String {
        let columns: Vec<String> = Self::columns()
            .iter()
            .map(|c| {
                let constraint = if c.primary_key {
                    " PRIMARY KEY"
                } else if c.nullable {
                    ""
                } else {
                    " NOT NULL"
                };
                format!("{} {}{}", c.name, c.sql_type, constraint)
            })
            .collect();
        format!("CREATE TABLE {} ({})", Self::TABLE, columns.join(", "))
    }

    /// `INSERT INTO people (id, name, email) VALUES (?1, ?2, ?3)`
    fn insert_sql() -> String {
        let names: Vec<&str> = Self::columns().iter().map(|c| c.name).collect();
        let params: Vec<String> = (1..=names.len()).map(|i| format!("?{i}")).collect();
        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            Self::TABLE,
            names.join(", "),
            params.join(", ")
        )
    }

    /// `SELECT id, name, email FROM people WHERE id = ?1`
    fn select_by_key_sql() -> String {
        let names: Vec<&str> = Self::columns().iter().map(|c| c.name).collect();
        let key = Self::columns()
            .iter()
            .find(|c| c.primary_key)
            .map_or("rowid", |c| c.name);
        format!(
            "SELECT {} FROM {} WHERE {} = ?1",
            names.join(", "),
            Self::TABLE,
            key
        )
    }
}
