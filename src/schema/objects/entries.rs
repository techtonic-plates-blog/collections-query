use chrono::{DateTime, Utc};
use entities::{fields, sea_orm_active_enums::DataTypes};
use juniper::{
    FieldResult, GraphQLInputObject, GraphQLObject, GraphQLScalar, GraphQLUnion, graphql_object,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter,
    Statement, TransactionTrait,
};
use typst_as_lib::TypstEngine;
use uuid::Uuid;

pub struct EntryRelation {
    pub from_entry_id: Uuid,
    pub to_entry_id: Uuid,
}

#[graphql_object(context = crate::state::AppData)]
impl EntryRelation {
    async fn from_entry(
        &self,
        context: &crate::state::AppData,
    ) -> juniper::FieldResult<Option<Entry>> {
        let db = &context.db;
        let entry = entities::entries::Entity::find_by_id(self.from_entry_id)
            .one(db)
            .await?;
        if let Some(entry) = entry {
            Ok(Some(Entry {
                id: entry.id,
                created_at: entry.created_at.and_utc(),
                collection_id: entry.collection_id,
                created_by: entry.created_by,
                name: entry.name,
            }))
        } else {
            Ok(None)
        }
    }

    async fn to_entry(
        &self,
        context: &crate::state::AppData,
    ) -> juniper::FieldResult<Option<Entry>> {
        let db = &context.db;
        let entry = entities::entries::Entity::find_by_id(self.to_entry_id)
            .one(db)
            .await?;
        if let Some(entry) = entry {
            Ok(Some(Entry {
                id: entry.id,
                created_at: entry.created_at.and_utc(),
                collection_id: entry.collection_id,
                created_by: entry.created_by,
                name: entry.name,
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(GraphQLObject)]
pub struct TypstText {
    pub raw: String,
    pub rendered: String,
}

#[derive(GraphQLObject)]
pub struct EntryObject {
    pub value: String,
}

#[derive(GraphQLObject)]
pub struct TextValue {
    pub value: Option<String>,
}

#[derive(GraphQLObject)]
pub struct BooleanValue {
    pub value: Option<bool>,
}

#[derive(GraphQLObject)]
pub struct NumberValue {
    pub value: Option<f64>,
}

#[derive(GraphQLObject)]
pub struct DateTimeValue {
    pub value: Option<DateTime<Utc>>,
}

#[derive(GraphQLObject)]
pub struct TextListValue {
    pub value: Vec<String>,
}

#[derive(GraphQLObject)]
pub struct NumberListValue {
    pub value: Vec<f64>,
}

#[derive(GraphQLUnion)]
#[graphql(context = crate::state::AppData)]
pub enum ValueType {
    Text(TextValue),
    TypstText(TypstText),
    Boolean(BooleanValue),
    Number(NumberValue),
    Relation(EntryRelation),
    DateTime(DateTimeValue),
    TextList(TextListValue),
    NumberList(NumberListValue),
    Object(EntryObject),
}

impl ValueType {
    async fn from_data_type(
        data_type: &entities::sea_orm_active_enums::DataTypes,
        entry_id: Uuid,
        field_id: Uuid,
        context: &crate::state::AppData,
    ) -> juniper::FieldResult<Option<ValueType>> {
        let db = &context.db;
        match data_type {
            entities::sea_orm_active_enums::DataTypes::Text => {
                let v = entities::entry_text_values::Entity::find()
                    .filter(entities::entry_text_values::Column::EntryId.eq(entry_id))
                    .filter(entities::entry_text_values::Column::FieldId.eq(field_id))
                    .one(db)
                    .await?;
                if let Some(v) = v {
                    Ok(Some(ValueType::Text(TextValue { value: v.value })))
                } else {
                    Ok(None)
                }
            }
            entities::sea_orm_active_enums::DataTypes::TypstText => {
                let v = entities::entry_typst_text_values::Entity::find()
                    .filter(entities::entry_typst_text_values::Column::EntryId.eq(entry_id))
                    .filter(entities::entry_typst_text_values::Column::FieldId.eq(field_id))
                    .one(db)
                    .await?;
                if let Some(v) = v {
                    Ok(Some(ValueType::TypstText(TypstText {
                        raw: v.raw,
                        rendered: v.rendered,
                    })))
                } else {
                    Ok(None)
                }
            }
            entities::sea_orm_active_enums::DataTypes::Boolean => {
                let v = entities::entry_boolean_values::Entity::find()
                    .filter(entities::entry_boolean_values::Column::EntryId.eq(entry_id))
                    .filter(entities::entry_boolean_values::Column::FieldId.eq(field_id))
                    .one(db)
                    .await?;
                if let Some(v) = v {
                    Ok(Some(ValueType::Boolean(BooleanValue { value: v.value })))
                } else {
                    Ok(None)
                }
            }
            entities::sea_orm_active_enums::DataTypes::Number => {
                let v = entities::entry_number_values::Entity::find()
                    .filter(entities::entry_number_values::Column::EntryId.eq(entry_id))
                    .filter(entities::entry_number_values::Column::FieldId.eq(field_id))
                    .one(db)
                    .await?;
                if let Some(v) = v {
                    Ok(Some(ValueType::Number(NumberValue { value: v.value })))
                } else {
                    Ok(None)
                }
            }
            entities::sea_orm_active_enums::DataTypes::Relation => {
                let v = entities::entry_relation_values::Entity::find()
                    .filter(entities::entry_relation_values::Column::FromEntryId.eq(entry_id))
                    .filter(entities::entry_relation_values::Column::FieldId.eq(field_id))
                    .one(db)
                    .await?;
                if let Some(v) = v {
                    Ok(Some(ValueType::Relation(EntryRelation {
                        from_entry_id: v.from_entry_id,
                        to_entry_id: v.to_entry_id,
                    })))
                } else {
                    Ok(None)
                }
            }
            entities::sea_orm_active_enums::DataTypes::DateTime => {
                let v = entities::entry_date_time_values::Entity::find()
                    .filter(entities::entry_date_time_values::Column::EntryId.eq(entry_id))
                    .filter(entities::entry_date_time_values::Column::FieldId.eq(field_id))
                    .one(db)
                    .await?;
                if let Some(v) = v {
                    Ok(Some(ValueType::DateTime(DateTimeValue {
                        value: v.value.map(|dt| dt.and_utc()),
                    })))
                } else {
                    Ok(None)
                }
            }
            entities::sea_orm_active_enums::DataTypes::TextList => {
                let model = entities::entry_text_list_values::Entity::find()
                    .filter(entities::entry_text_list_values::Column::EntryId.eq(entry_id))
                    .filter(entities::entry_text_list_values::Column::FieldId.eq(field_id))
                    .one(db)
                    .await?;
                if let Some(value) = model {
                    Ok(Some(ValueType::TextList(TextListValue {
                        value: value.value.unwrap_or_default(),
                    })))
                } else {
                    Ok(None)
                }
            }
            entities::sea_orm_active_enums::DataTypes::NumberList => {
                let model = entities::entry_number_list_values::Entity::find()
                    .filter(entities::entry_number_list_values::Column::EntryId.eq(entry_id))
                    .filter(entities::entry_number_list_values::Column::FieldId.eq(field_id))
                    .one(db)
                    .await?;
                if let Some(value) = model {
                    Ok(Some(ValueType::NumberList(NumberListValue {
                        value: value.value.unwrap_or_default(),
                    })))
                } else {
                    Ok(None)
                }
            }
            entities::sea_orm_active_enums::DataTypes::Object => {
                let v = entities::entry_object_values::Entity::find()
                    .filter(entities::entry_object_values::Column::EntryId.eq(entry_id))
                    .filter(entities::entry_object_values::Column::FieldId.eq(field_id))
                    .one(db)
                    .await?;
                if let Some(v) = v {
                    Ok(Some(ValueType::Object(EntryObject {
                        value: v.value.to_string(),
                    })))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

#[derive(GraphQLObject)]
#[graphql(context = crate::state::AppData)]
pub struct FieldValue {
    pub field: super::collection::Field,
    pub value: ValueType,
}
pub struct Entry {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub collection_id: Uuid,
    pub created_by: Uuid,
    pub name: String,
}

#[graphql_object(context = crate::state::AppData)]
impl Entry {
    fn id(&self) -> Uuid {
        self.id
    }
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    fn created_by(&self) -> Uuid {
        self.created_by
    }
    fn name(&self) -> &str {
        &self.name
    }
    async fn values(
        &self,
        context: &crate::state::AppData,
    ) -> juniper::FieldResult<Vec<FieldValue>> {
        let db = &context.db;

        let fields = entities::fields::Entity::find()
            .filter(entities::fields::Column::CollectionId.eq(self.collection_id))
            .all(db)
            .await?;

        let mut values = vec![];

        for field in fields {
            let v = ValueType::from_data_type(&field.data_type, self.id, field.id, context).await?;
            if let Some(v) = v {
                values.push(FieldValue { field: super::collection::Field {
                    id: field.id,
                    collection_id: field.collection_id,
                    name: field.name,
                    data_type: field.data_type,
                    created_at: field.created_at.and_utc(),
                }, value: v });
            }
        }

        Ok(values)
    }
}
