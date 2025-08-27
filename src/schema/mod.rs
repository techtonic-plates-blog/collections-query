use juniper::RootNode;
use crate::state::AppData;

mod query;
mod objects;

pub type Schema<'a> = RootNode<'a, query::Query, juniper::EmptyMutation<AppData>, juniper::EmptySubscription<AppData>>;

pub fn schema() -> Schema<'static> {
    Schema::new(query::Query, juniper::EmptyMutation::new(), juniper::EmptySubscription::new())
}