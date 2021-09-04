//use diesel::backend::{BindCollector, QueryBuilder};
// use diesel::impl_query_id; // deprecated since 1.1.0
use diesel::pg::Pg;
use diesel::query_builder::{AsQuery, AstPass, Query, QueryFragment};
use diesel::query_dsl::limit_dsl::LimitDsl;
use diesel::sql_types::BigInt;
use diesel::{dsl, Expression};
use diesel::{PgConnection, QueryDsl, QueryResult, RunQueryDsl};
use diesel::{QueryId, Queryable};
use diesel_demo::schema::posts;

// Implement `QueryFragment`
impl<T> QueryFragment<Pg> for PaginatedResult<T>
where
    T: QueryFragment<Pg>,
    PaginatedResult<T>: LimitDsl,
{
    fn walk_ast(&self, mut out: AstPass<Pg>) -> QueryResult<()> {
        out.push_sql("select *, count(*) over () from (");
        self.query.walk_ast(out.reborrow())?;
        out.push_sql(") LIMIT ");
        out.push_bind_param(&self.limit(self.per_page))?;
        out.push_sql(" OFFSET ");
        out.push_bind_param::<BigInt, _>(&self.offset())?;
        Ok(())
    }
}

// Whenever you implement `QueryFragment` you need to implement `QueryId`
impl<T: Query> Query for PaginatedResult<T> {
    type SqlType = (T::SqlType, BigInt);
}

impl<T> RunQueryDsl<PgConnection> for PaginatedResult<T> {}

// FIXME: maybe implement this? but not this way?
// impl<T> QueryDsl for Paginated<T> {}
impl<T: LimitDsl + LimitDsl<Output = T>> LimitDsl for PaginatedResult<T> {
    type Output = PaginatedResult<dsl::Limit<T>>;

    fn limit(self, limit: i64) -> Self::Output {
        limit
    }
}

// Using `trait Paginate` to implement all that
pub trait Paginate: AsQuery + Sized {
    fn paginate(self, page: i64) -> PaginatedResult<Self::Query> {
        PaginatedResult {
            query: self.as_query(),
            page,
            per_page: DEFAULT_PER_PAGE,
        }
    }
}

impl<T: AsQuery> Paginate for T {}
const DEFAULT_PER_PAGE: i64 = 10;

#[derive(Queryable, QueryId, Debug)]
pub struct PaginatedResult<T> {
    query: T,
    page: i64,
    per_page: i64,
}

impl<T> PaginatedResult<T> {
    pub fn per_page(self, per_page: i64) -> Self {
        PaginatedResult { per_page, ..self }
    }
}

fn main() {
    let second = posts::table.paginate(3).per_page(2);
    println!("count hay {:?}", &second);
}
