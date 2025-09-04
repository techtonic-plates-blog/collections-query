use juniper::FieldResult;

use super::objects::collection::Collection;
use crate::state::AppData;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, prelude::Expr};

#[derive(Clone, Copy, Debug)]
pub struct Query;

#[derive(juniper::GraphQLObject)]
#[graphql(context = crate::state::AppData)]
pub struct CollectionsPage {
    pub items: Vec<Collection>,
    pub num_pages: i32,
    pub num_items: i32,
    pub index: i32,
    pub size: i32,
}

#[juniper::graphql_object(context = crate::state::AppData)]
impl Query {
    fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    async fn collections(
        ctx: &AppData,
        collection_name: Option<String>,
        page: Option<i32>,
        page_size: Option<i32>,
        created_after: Option<chrono::DateTime<chrono::Utc>>,
        created_before: Option<chrono::DateTime<chrono::Utc>>,

    ) -> FieldResult<CollectionsPage> {
        let db = &ctx.db;
        let mut query = entities::collections::Entity::find();

        let page_num = page.unwrap_or(1).max(1);
        let page_size = page_size.unwrap_or(10).max(1).min(100);

        if let Some(name) = collection_name {
            query = query.filter(Expr::cust_with_values(
                "to_tsvector('english', name) @@ to_tsquery('english', $1)",
                vec![sea_orm::Value::String(Some(Box::new(name)))],
            ));
        }
        if let Some(after) = created_after {
            query = query.filter(entities::collections::Column::CreatedAt.gte(after));
        }
        if let Some(before) = created_before {
            query = query.filter(entities::collections::Column::CreatedAt.lte(before));
        }

        let page = query.paginate(db, page_size as u64);
        let items = page.fetch_page(page_num as u64 - 1).await?;
        let items_and_pages = page.num_items_and_pages().await?;
        let collections = items
            .into_iter()
            .map(|c| Collection {
                id: c.id,
                name: c.name,
                created_at: c.created_at.and_utc(),
                created_by: c.created_by,
            })
            .collect();

        let collections_page = CollectionsPage {
            items: collections,
            num_pages: items_and_pages.number_of_pages as i32,
            num_items: items_and_pages.number_of_items as i32,
            index: page_num,
            size: page_size,
        };

        Ok(collections_page)
    }

    async fn collection(ctx: &AppData, name: String) -> FieldResult<Option<Collection>> {
        let db = &ctx.db;
        let collection = entities::collections::Entity::find()
            .filter(entities::collections::Column::Name.eq(name))
            .one(db)
            .await?;

        if let Some(c) = collection {
            Ok(Some(Collection {
                id: c.id,
                name: c.name,
                created_at: c.created_at.and_utc(),
                created_by: c.created_by,
            }))
        } else {
            Ok(None)
        }
    }
}
