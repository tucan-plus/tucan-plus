
use crate::schema::*;
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    prelude::*,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(
    Debug, PartialEq, FromSqlRow, AsExpression, Eq, Copy, Clone, Hash, Serialize, Deserialize,
)]
#[diesel(sql_type = Text)]
pub enum Semester {
    Sommersemester,
    Wintersemester,
}

impl std::fmt::Display for Semester {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Semester::Sommersemester => write!(f, "Sommersemester"),
            Semester::Wintersemester => write!(f, "Wintersemester"),
        }
    }
}

impl ToSql<Text, diesel::sqlite::Sqlite> for Semester {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::sqlite::Sqlite>) -> serialize::Result {
        out.set_value(match self {
            Semester::Sommersemester => "s",
            Semester::Wintersemester => "w",
        });
        Ok(IsNull::No)
    }
}

impl<DB> FromSql<Text, DB> for Semester
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        match String::from_sql(bytes)?.as_str() {
            "s" => Ok(Semester::Sommersemester),
            "w" => Ok(Semester::Wintersemester),
            x => Err(format!("Unrecognized variant {}", x).into()),
        }
    }
}

#[derive(
    Insertable, Queryable, Selectable, Clone, Debug, Serialize, Deserialize, PartialEq, Eq,
)]
#[diesel(table_name = anmeldungen_plan)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(treat_none_as_default_value = false)]
pub struct Anmeldung {
    pub course_of_study: String,
    pub url: String, // AnmeldungRequest
    pub name: String,
    pub parent: Option<String>,
    pub min_cp: i32,
    pub max_cp: Option<i32>,
    pub min_modules: i32,
    pub max_modules: Option<i32>,
}

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq, Copy, Clone, Serialize, Deserialize)]
#[diesel(sql_type = Text)]
pub enum State {
    NotPlanned,
    MaybePlanned,
    Planned,
    Done,
}

impl ToSql<Text, diesel::sqlite::Sqlite> for State {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, diesel::sqlite::Sqlite>) -> serialize::Result {
        out.set_value(match self {
            Self::NotPlanned => "not_planned",
            Self::MaybePlanned => "maybe_planned",
            Self::Planned => "planned",
            Self::Done => "done",
        });
        Ok(IsNull::No)
    }
}

impl<DB> FromSql<Text, DB> for State
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        match String::from_sql(bytes)?.as_str() {
            "not_planned" => Ok(Self::NotPlanned),
            "maybe_planned" => Ok(Self::MaybePlanned),
            "planned" => Ok(Self::Planned),
            "done" => Ok(Self::Done),
            x => Err(format!("Unrecognized variant {}", x).into()),
        }
    }
}

#[derive(
    Insertable,
    Queryable,
    Selectable,
    Clone,
    PartialEq,
    Eq,
    Debug,
    AsChangeset,
    Identifiable,
    Serialize,
    Deserialize,
)]
#[diesel(table_name = anmeldungen_entries)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(course_of_study, available_semester, anmeldung, id))]
#[diesel(treat_none_as_default_value = false)]
#[diesel(treat_none_as_null = true)]
pub struct AnmeldungEntry {
    pub course_of_study: String,
    pub available_semester: Semester,
    pub anmeldung: String,
    pub module_url: Option<String>,
    pub id: String,
    pub name: String,
    pub credits: i32,
    pub state: State,
    pub semester: Option<Semester>,
    pub year: Option<i32>,
}

#[derive(
    Insertable,
    Queryable,
    Selectable,
    Clone,
    PartialEq,
    Debug,
    AsChangeset,
    Identifiable,
    Serialize,
    Deserialize,
)]
#[diesel(table_name = cache)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(key))]
#[diesel(treat_none_as_default_value = false)]
#[diesel(treat_none_as_null = true)]
pub struct CacheEntry {
    pub key: String,
    pub value: String,
    pub updated: OffsetDateTime,
}
