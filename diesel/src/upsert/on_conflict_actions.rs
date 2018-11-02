use backend::{Backend, SupportsOnConflictClause};
use expression::{AppearsOnTable, Expression};
use query_builder::*;
use query_source::*;
use result::QueryResult;

/// Represents `excluded.column` in an `ON CONFLICT DO UPDATE` clause.
pub fn excluded<T>(excluded: T) -> Excluded<T> {
    Excluded(excluded)
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct DoNothing;

impl<DB: Backend + SupportsOnConflictClause> QueryFragment<DB> for DoNothing {
    fn walk_ast(&self, mut out: AstPass<DB>) -> QueryResult<()> {
        out.push_sql(" DO NOTHING");
        Ok(())
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct DoUpdate<T> {
    changeset: T,
}

impl<T> DoUpdate<T> {
    pub(crate) fn new(changeset: T) -> Self {
        DoUpdate { changeset }
    }
}

impl<DB, T> QueryFragment<DB> for DoUpdate<T>
where
    DB: Backend + SupportsOnConflictClause,
    T: QueryFragment<DB>,
{
    fn walk_ast(&self, mut out: AstPass<DB>) -> QueryResult<()> {
        out.unsafe_to_cache_prepared();
        if self.changeset.is_noop()? {
            out.push_sql(" DO NOTHING");
        } else {
            out.push_sql(" DO UPDATE SET ");
            self.changeset.walk_ast(out.reborrow())?;
        }
        Ok(())
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct Excluded<T>(T);

impl<DB, T> QueryFragment<DB> for Excluded<T>
where
    DB: Backend + SupportsOnConflictClause,
    T: Column,
{
    fn walk_ast(&self, mut out: AstPass<DB>) -> QueryResult<()> {
        out.push_sql("excluded.");
        try!(out.push_identifier(T::NAME));
        Ok(())
    }
}

impl<T> Expression for Excluded<T>
where
    T: Expression,
{
    type SqlType = T::SqlType;
}

impl<T> AppearsOnTable<T::Table> for Excluded<T>
where
    T: Column,
    Excluded<T>: Expression,
{
}
