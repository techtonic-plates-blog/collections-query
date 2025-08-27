use crate::{schema::objects::entries::Entry, state::AppData};
use chrono::{DateTime, Utc};
use entities::{collections, fields, sea_orm_active_enums::DataTypes};
use juniper::{FieldResult, GraphQLInputObject, GraphQLObject, graphql_object};
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

pub struct Collection {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(GraphQLObject)]
pub struct Field {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub name: String,
    pub data_type: DataTypes,
    pub created_at: DateTime<Utc>,
}

#[graphql_object(context = crate::state::AppData)]
impl Collection {
    fn id(&self) -> Uuid {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    fn created_by(&self) -> Uuid {
        self.created_by
    }

    async fn fields(&self, ctx: &AppData) -> FieldResult<Vec<Field>> {
        let db = &ctx.db;
        let fields = entities::fields::Entity::find()
            .filter(entities::fields::Column::CollectionId.eq(self.id))
            .all(db)
            .await?
            .into_iter()
            .map(|f| Field {
                id: f.id,
                collection_id: f.collection_id,
                name: f.name,
                data_type: f.data_type,
                created_at: f.created_at.and_utc(),
            })
            .collect();
        Ok(fields)
    }

    async fn entries(&self, ctx: &AppData) -> FieldResult<Vec<Entry>> {
        let db = &ctx.db;
        let entries = entities::entries::Entity::find()
            .filter(entities::entries::Column::CollectionId.eq(self.id))
            .all(db)
            .await?
            .into_iter()
            .map(|e| Entry {
                id: e.id,
                created_at: e.created_at.and_utc(),
                collection_id: e.collection_id,
                created_by: e.created_by,
                name: e.name,
            })
            .collect();
        Ok(entries)
    }
}
