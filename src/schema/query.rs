
use juniper::{ FieldResult};

use sea_orm::{EntityTrait};
use crate::state::AppData;
use super::objects::collection::Collection;

#[derive(Clone, Copy, Debug)]
pub struct Query;


#[juniper::graphql_object(context = crate::state::AppData)]
impl Query {
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    async fn collections(ctx: &AppData) -> FieldResult<Vec<Collection>> {
        let db = &ctx.db;
        let collections = entities::collections::Entity::find()
            .all(db)
            .await?
            .into_iter()
            .map(|c| Collection {
                id: c.id,
                name: c.name,
                created_at: c.created_at.and_utc(),
                created_by: c.created_by,
            })
            .collect();
        Ok(collections)
    }
}
