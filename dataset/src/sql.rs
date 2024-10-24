use sqlx::{self, Arguments};

pub trait ToArguments<DB: sqlx::Database>: crate::MetaModel + Send {
    fn to_arguments<'m:'q,'q>(&'m self) -> sqlx::Result<(<DB as sqlx::Database>::Arguments<'q>, usize)>;
}

pub trait SqlxArgumentsExtended<'q, DB: sqlx::Database>: sqlx::Arguments<'q> {
    fn start_args(capacity: usize) -> sqlx::Result<Self, sqlx::Error>;
    fn arg<T>(self, value: T) -> sqlx::Result<Self, sqlx::Error> where T: sqlx::Encode<'q, DB> + sqlx::Type<sqlx::Sqlite> + 'q;
    fn finish_args<M: crate::MetaModel>(self, model: &M, capacity: usize) -> sqlx::Result<(Self, usize), sqlx::Error>;
}

impl<'q> SqlxArgumentsExtended<'q, sqlx::sqlite::Sqlite> for sqlx::sqlite::SqliteArguments<'q> {
    fn start_args(capacity: usize) -> sqlx::Result<Self, sqlx::Error> {
        let mut args = sqlx::sqlite::SqliteArguments::default();
        args.reserve(capacity, capacity*64);
        Ok(args)
    }

    fn arg<T>(mut self, value: T) -> sqlx::Result<Self, sqlx::Error> where T: sqlx::Encode<'q, sqlx::sqlite::Sqlite> + sqlx::Type<sqlx::Sqlite> + 'q {
        self.add(value).map_err(|e| sqlx::Error::Encode(e))?;
        Ok(self)
    }

    fn finish_args<M: crate::MetaModel>(self, model: &M, capacity: usize) -> sqlx::Result<(Self, usize), sqlx::Error> {
        let mut args = self
            .arg(model.user_id().sql())?
            .arg(model.time_created())?
            .arg(model.time_modified())?;

        if let Some(id) = model.meta().id().best_valid_sql() {
            args = args.arg(id)?;
        }

        Ok((args, capacity))
    }
}