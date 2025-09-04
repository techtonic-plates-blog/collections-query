use crate::{schema::objects::entries::Entry, state::AppData};
use chrono::{DateTime, Utc};
use entities::sea_orm_active_enums::DataTypes;
use juniper::{graphql_object, FieldResult, GraphQLEnum, GraphQLInputObject, GraphQLObject, Value};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
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

#[derive(GraphQLEnum)]
pub enum TextComparison {
    Eq,
    Neq,
    Contains,
    StartsWith,
    EndsWith,
}

#[derive(GraphQLEnum)]
pub enum NumberComparison {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(GraphQLEnum)]
pub enum BooleanComparison {
    Eq,
    Neq,
}

#[derive(GraphQLEnum)]
pub enum DateTimeComparison {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
}

#[derive(GraphQLEnum)]
pub enum ListComparison {
    Contains,
    ContainsAll,
    ContainsAny,
    IsEmpty,
    IsNotEmpty,
}

#[derive(GraphQLEnum)]
pub enum RelationComparison {
    ConnectedTo,
    NotConnectedTo,
    HasConnections,
    HasNoConnections,
}

#[derive(GraphQLEnum)]
pub enum ObjectComparison {
    HasProperty,
    PropertyEquals,
    PropertyContains,
    IsEmpty,
    IsNotEmpty,
}

// Specific filter types for each data type
#[derive(GraphQLInputObject)]
pub struct TextFilter {
    pub field_name: String,
    pub comparison: TextComparison,
    pub value: String,
}

#[derive(GraphQLInputObject)]
pub struct NumberFilter {
    pub field_name: String,
    pub comparison: NumberComparison,
    pub value: f64,
}

#[derive(GraphQLInputObject)]
pub struct BooleanFilter {
    pub field_name: String,
    pub comparison: BooleanComparison,
    pub value: bool,
}

#[derive(GraphQLInputObject)]
pub struct DateTimeFilter {
    pub field_name: String,
    pub comparison: DateTimeComparison,
    pub value: String, // ISO 8601 format
}

#[derive(GraphQLInputObject)]
pub struct ListFilter {
    pub field_name: String,
    pub comparison: ListComparison,
    pub values: Option<Vec<String>>, // For ContainsAll/ContainsAny
}

#[derive(GraphQLInputObject)]
pub struct RelationFilter {
    pub field_name: String,
    pub comparison: RelationComparison,
    pub target_entry_id: Option<String>, // UUID as string for ConnectedTo/NotConnectedTo
}

#[derive(GraphQLInputObject)]
pub struct ObjectFilter {
    pub field_name: String,
    pub comparison: ObjectComparison,
    pub property_path: Option<String>, // JSON path like "address.city"
    pub property_value: Option<String>, // Value to compare against
}

// Main filter input that accepts specific filter types
#[derive(GraphQLInputObject)]
pub struct EntryFilters {
    pub text_filters: Option<Vec<TextFilter>>,
    pub number_filters: Option<Vec<NumberFilter>>,
    pub boolean_filters: Option<Vec<BooleanFilter>>,
    pub date_time_filters: Option<Vec<DateTimeFilter>>,
    pub list_filters: Option<Vec<ListFilter>>,
    pub relation_filters: Option<Vec<RelationFilter>>,
    pub object_filters: Option<Vec<ObjectFilter>>,
}

#[derive(GraphQLEnum)]
pub enum EntryOrderBy {
    Asc,
    Desc
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

    async fn entries(&self, ctx: &AppData, filters: Option<EntryFilters>, order_by: Option<EntryOrderBy>) -> FieldResult<Vec<Entry>> {
        let db = &ctx.db;
        
        let order_by = order_by.unwrap_or(EntryOrderBy::Asc);
        let order_by = match order_by {
            EntryOrderBy::Asc => sea_orm::Order::Asc,
            EntryOrderBy::Desc => sea_orm::Order::Desc,
        };

        let fields = self.fields(ctx).await?;

        // Start with the base query for entries in this collection
        let mut base_query = entities::entries::Entity::find()
            .filter(entities::entries::Column::CollectionId.eq(self.id));

        // Apply filters if provided
        if let Some(filters) = filters {
            base_query = self.apply_typed_filters(base_query, &fields, filters).await?;
        }

        let entries = base_query
            .order_by(entities::entries::Column::CreatedAt, order_by) // Order by creation date
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

    async fn entry(&self, ctx: &AppData, name: String) -> FieldResult<Entry> {
        let db = &ctx.db;
        let entry = entities::entries::Entity::find()
            .filter(entities::entries::Column::CollectionId.eq(self.id))
            .filter(entities::entries::Column::Name.eq(name))
            .one(db)
            .await?;

        if let Some(e) = entry {
            Ok(Entry {
                id: e.id,
                created_at: e.created_at.and_utc(),
                collection_id: e.collection_id,
                created_by: e.created_by,
                name: e.name,
            })
        } else {
            Err(juniper::FieldError::new(
                "Entry not found".to_string(),
                Value::null(),
            ))
        }
    }
}

impl Collection {
    // Apply all typed filters to the query
    async fn apply_typed_filters(
        &self,
        mut query: sea_orm::Select<entities::entries::Entity>,
        fields: &[Field],
        filters: EntryFilters,
    ) -> FieldResult<sea_orm::Select<entities::entries::Entity>> {
        
        // Apply text filters (includes both Text and TypstText)
        if let Some(text_filters) = filters.text_filters {
            for filter in text_filters {
                query = self.apply_text_filter(query, fields, filter)?;
            }
        }

        // Apply number filters
        if let Some(number_filters) = filters.number_filters {
            for filter in number_filters {
                query = self.apply_number_filter(query, fields, filter)?;
            }
        }

        // Apply boolean filters
        if let Some(boolean_filters) = filters.boolean_filters {
            for filter in boolean_filters {
                query = self.apply_boolean_filter(query, fields, filter)?;
            }
        }

        // Apply datetime filters
        if let Some(datetime_filters) = filters.date_time_filters {
            for filter in datetime_filters {
                query = self.apply_datetime_filter(query, fields, filter)?;
            }
        }

        // Apply list filters
        if let Some(list_filters) = filters.list_filters {
            for filter in list_filters {
                query = self.apply_list_filter(query, fields, filter)?;
            }
        }

        // Apply relation filters
        if let Some(relation_filters) = filters.relation_filters {
            for filter in relation_filters {
                query = self.apply_relation_filter(query, fields, filter)?;
            }
        }

        // Apply object filters
        if let Some(object_filters) = filters.object_filters {
            for filter in object_filters {
                query = self.apply_object_filter(query, fields, filter)?;
            }
        }

        Ok(query)
    }

    // Validate field exists and has correct data type
    fn validate_field<'a>(&self, fields: &'a [Field], field_name: &str, expected_types: &[DataTypes]) -> FieldResult<&'a Field> {
        let field = fields.iter().find(|f| f.name == field_name)
            .ok_or_else(|| juniper::FieldError::new(
                format!("Field '{}' does not exist in collection", field_name),
                Value::null(),
            ))?;

        if !expected_types.contains(&field.data_type) {
            return Err(juniper::FieldError::new(
                format!("Field '{}' has type {:?}, expected one of {:?}", 
                    field_name, field.data_type, expected_types),
                Value::null(),
            ));
        }

        Ok(field)
    }

    // Apply text filter (includes both Text and TypstText)
    fn apply_text_filter(
        &self,
        mut query: sea_orm::Select<entities::entries::Entity>,
        fields: &[Field],
        filter: TextFilter,
    ) -> FieldResult<sea_orm::Select<entities::entries::Entity>> {
        let field = self.validate_field(fields, &filter.field_name, 
            &[DataTypes::Text, DataTypes::TypstText])?;

        // Both Text and TypstText use the same table
        query = query.inner_join(entities::entry_text_values::Entity)
            .filter(entities::entry_text_values::Column::FieldId.eq(field.id));

        match filter.comparison {
            TextComparison::Eq => {
                query = query.filter(entities::entry_text_values::Column::Value.eq(&filter.value));
            }
            TextComparison::Neq => {
                query = query.filter(entities::entry_text_values::Column::Value.ne(&filter.value));
            }
            TextComparison::Contains => {
                let pattern = format!("%{}%", filter.value);
                query = query.filter(entities::entry_text_values::Column::Value.like(pattern));
            }
            TextComparison::StartsWith => {
                let pattern = format!("{}%", filter.value);
                query = query.filter(entities::entry_text_values::Column::Value.like(pattern));
            }
            TextComparison::EndsWith => {
                let pattern = format!("%{}", filter.value);
                query = query.filter(entities::entry_text_values::Column::Value.like(pattern));
            }
        }

        Ok(query)
    }

    // Apply number filter
    fn apply_number_filter(
        &self,
        mut query: sea_orm::Select<entities::entries::Entity>,
        fields: &[Field],
        filter: NumberFilter,
    ) -> FieldResult<sea_orm::Select<entities::entries::Entity>> {
        let field = self.validate_field(fields, &filter.field_name, &[DataTypes::Number])?;

        query = query.inner_join(entities::entry_number_values::Entity)
            .filter(entities::entry_number_values::Column::FieldId.eq(field.id));

        match filter.comparison {
            NumberComparison::Eq => {
                query = query.filter(entities::entry_number_values::Column::Value.eq(filter.value));
            }
            NumberComparison::Neq => {
                query = query.filter(entities::entry_number_values::Column::Value.ne(filter.value));
            }
            NumberComparison::Gt => {
                query = query.filter(entities::entry_number_values::Column::Value.gt(filter.value));
            }
            NumberComparison::Gte => {
                query = query.filter(entities::entry_number_values::Column::Value.gte(filter.value));
            }
            NumberComparison::Lt => {
                query = query.filter(entities::entry_number_values::Column::Value.lt(filter.value));
            }
            NumberComparison::Lte => {
                query = query.filter(entities::entry_number_values::Column::Value.lte(filter.value));
            }
        }

        Ok(query)
    }

    // Apply boolean filter
    fn apply_boolean_filter(
        &self,
        mut query: sea_orm::Select<entities::entries::Entity>,
        fields: &[Field],
        filter: BooleanFilter,
    ) -> FieldResult<sea_orm::Select<entities::entries::Entity>> {
        let field = self.validate_field(fields, &filter.field_name, &[DataTypes::Boolean])?;

        query = query.inner_join(entities::entry_boolean_values::Entity)
            .filter(entities::entry_boolean_values::Column::FieldId.eq(field.id));

        match filter.comparison {
            BooleanComparison::Eq => {
                query = query.filter(entities::entry_boolean_values::Column::Value.eq(filter.value));
            }
            BooleanComparison::Neq => {
                query = query.filter(entities::entry_boolean_values::Column::Value.ne(filter.value));
            }
        }

        Ok(query)
    }

    // Apply datetime filter
    fn apply_datetime_filter(
        &self,
        mut query: sea_orm::Select<entities::entries::Entity>,
        fields: &[Field],
        filter: DateTimeFilter,
    ) -> FieldResult<sea_orm::Select<entities::entries::Entity>> {
        let field = self.validate_field(fields, &filter.field_name, &[DataTypes::DateTime])?;

        query = query.inner_join(entities::entry_date_time_values::Entity)
            .filter(entities::entry_date_time_values::Column::FieldId.eq(field.id));

        // Note: You might want to parse the ISO 8601 string to a proper DateTime here
        match filter.comparison {
            DateTimeComparison::Eq => {
                query = query.filter(entities::entry_date_time_values::Column::Value.eq(&filter.value));
            }
            DateTimeComparison::Neq => {
                query = query.filter(entities::entry_date_time_values::Column::Value.ne(&filter.value));
            }
            DateTimeComparison::Gt => {
                query = query.filter(entities::entry_date_time_values::Column::Value.gt(&filter.value));
            }
            DateTimeComparison::Gte => {
                query = query.filter(entities::entry_date_time_values::Column::Value.gte(&filter.value));
            }
            DateTimeComparison::Lt => {
                query = query.filter(entities::entry_date_time_values::Column::Value.lt(&filter.value));
            }
            DateTimeComparison::Lte => {
                query = query.filter(entities::entry_date_time_values::Column::Value.lte(&filter.value));
            }
        }

        Ok(query)
    }

    // Apply list filter
    fn apply_list_filter(
        &self,
        mut query: sea_orm::Select<entities::entries::Entity>,
        fields: &[Field],
        filter: ListFilter,
    ) -> FieldResult<sea_orm::Select<entities::entries::Entity>> {
        let field = self.validate_field(fields, &filter.field_name, 
            &[DataTypes::TextList, DataTypes::NumberList])?;

        match field.data_type {
            DataTypes::TextList => {
                query = query.inner_join(entities::entry_text_list_values::Entity)
                    .filter(entities::entry_text_list_values::Column::FieldId.eq(field.id));

                match filter.comparison {
                    ListComparison::Contains => {
                        if let Some(values) = &filter.values {
                            if let Some(value) = values.first() {
                                let pattern = format!("%{}%", value);
                                query = query.filter(entities::entry_text_list_values::Column::Value.like(pattern));
                            }
                        }
                    }
                    ListComparison::IsEmpty => {
                        query = query.filter(entities::entry_text_list_values::Column::Value.is_null());
                    }
                    ListComparison::IsNotEmpty => {
                        query = query.filter(entities::entry_text_list_values::Column::Value.is_not_null());
                    }
                    _ => {
                        return Err(juniper::FieldError::new(
                            "ContainsAll and ContainsAny not yet implemented for lists".to_string(),
                            Value::null(),
                        ));
                    }
                }
            }
            DataTypes::NumberList => {
                query = query.inner_join(entities::entry_number_list_values::Entity)
                    .filter(entities::entry_number_list_values::Column::FieldId.eq(field.id));

                match filter.comparison {
                    ListComparison::IsEmpty => {
                        query = query.filter(entities::entry_number_list_values::Column::Value.is_null());
                    }
                    ListComparison::IsNotEmpty => {
                        query = query.filter(entities::entry_number_list_values::Column::Value.is_not_null());
                    }
                    _ => {
                        return Err(juniper::FieldError::new(
                            "Complex list operations not yet implemented for number lists".to_string(),
                            Value::null(),
                        ));
                    }
                }
            }
            _ => unreachable!(), // validate_field ensures correct types
        }

        Ok(query)
    }

    // Apply relation filter
    fn apply_relation_filter(
        &self,
        mut query: sea_orm::Select<entities::entries::Entity>,
        fields: &[Field],
        filter: RelationFilter,
    ) -> FieldResult<sea_orm::Select<entities::entries::Entity>> {
        let field = self.validate_field(fields, &filter.field_name, &[DataTypes::Relation])?;

        match filter.comparison {
            RelationComparison::ConnectedTo => {
                if let Some(target_id) = &filter.target_entry_id {
                    if let Ok(target_uuid) = uuid::Uuid::parse_str(target_id) {
                        // Use EXISTS subquery to check if the entry has a relation to the target
                        query = query.filter(
                            sea_orm::Condition::all().add(
                                sea_orm::sea_query::Expr::exists(
                                    sea_orm::sea_query::Query::select()
                                        .column(entities::entry_relation_values::Column::FromEntryId)
                                        .from(entities::entry_relation_values::Entity)
                                        .and_where(sea_orm::sea_query::Expr::col((
                                            entities::entry_relation_values::Entity, 
                                            entities::entry_relation_values::Column::FromEntryId
                                        )).equals((
                                            entities::entries::Entity, 
                                            entities::entries::Column::Id
                                        )))
                                        .and_where(entities::entry_relation_values::Column::FieldId
                                            .eq(field.id))
                                        .and_where(entities::entry_relation_values::Column::ToEntryId
                                            .eq(target_uuid))
                                        .to_owned()
                                )
                            )
                        );
                    } else {
                        return Err(juniper::FieldError::new(
                            format!("Invalid UUID format: '{}'", target_id),
                            Value::null(),
                        ));
                    }
                } else {
                    return Err(juniper::FieldError::new(
                        "target_entry_id is required for ConnectedTo comparison".to_string(),
                        Value::null(),
                    ));
                }
            }
            RelationComparison::NotConnectedTo => {
                if let Some(target_id) = &filter.target_entry_id {
                    if let Ok(target_uuid) = uuid::Uuid::parse_str(target_id) {
                        // Use NOT EXISTS subquery to check if the entry is NOT connected to the target
                        query = query.filter(
                            sea_orm::Condition::all().add(
                                sea_orm::sea_query::Expr::exists(
                                    sea_orm::sea_query::Query::select()
                                        .column(entities::entry_relation_values::Column::FromEntryId)
                                        .from(entities::entry_relation_values::Entity)
                                        .and_where(sea_orm::sea_query::Expr::col((
                                            entities::entry_relation_values::Entity, 
                                            entities::entry_relation_values::Column::FromEntryId
                                        )).equals((
                                            entities::entries::Entity, 
                                            entities::entries::Column::Id
                                        )))
                                        .and_where(entities::entry_relation_values::Column::FieldId
                                            .eq(field.id))
                                        .and_where(entities::entry_relation_values::Column::ToEntryId
                                            .eq(target_uuid))
                                        .to_owned()
                                ).not()
                            )
                        );
                    } else {
                        return Err(juniper::FieldError::new(
                            format!("Invalid UUID format: '{}'", target_id),
                            Value::null(),
                        ));
                    }
                } else {
                    return Err(juniper::FieldError::new(
                        "target_entry_id is required for NotConnectedTo comparison".to_string(),
                        Value::null(),
                    ));
                }
            }
            RelationComparison::HasConnections => {
                // Use EXISTS subquery to check if the entry has any relations for this field
                query = query.filter(
                    sea_orm::Condition::all().add(
                        sea_orm::sea_query::Expr::exists(
                            sea_orm::sea_query::Query::select()
                                .column(entities::entry_relation_values::Column::FromEntryId)
                                .from(entities::entry_relation_values::Entity)
                                .and_where(sea_orm::sea_query::Expr::col((
                                    entities::entry_relation_values::Entity, 
                                    entities::entry_relation_values::Column::FromEntryId
                                )).equals((
                                    entities::entries::Entity, 
                                    entities::entries::Column::Id
                                )))
                                .and_where(entities::entry_relation_values::Column::FieldId
                                    .eq(field.id))
                                .to_owned()
                        )
                    )
                );
            }
            RelationComparison::HasNoConnections => {
                // Use NOT EXISTS subquery to check if the entry has no relations for this field
                query = query.filter(
                    sea_orm::Condition::all().add(
                        sea_orm::sea_query::Expr::exists(
                            sea_orm::sea_query::Query::select()
                                .column(entities::entry_relation_values::Column::FromEntryId)
                                .from(entities::entry_relation_values::Entity)
                                .and_where(sea_orm::sea_query::Expr::col((
                                    entities::entry_relation_values::Entity, 
                                    entities::entry_relation_values::Column::FromEntryId
                                )).equals((
                                    entities::entries::Entity, 
                                    entities::entries::Column::Id
                                )))
                                .and_where(entities::entry_relation_values::Column::FieldId
                                    .eq(field.id))
                                .to_owned()
                        ).not()
                    )
                );
            }
        }

        Ok(query)
    }

    // Apply object filter
    fn apply_object_filter(
        &self,
        mut query: sea_orm::Select<entities::entries::Entity>,
        fields: &[Field],
        filter: ObjectFilter,
    ) -> FieldResult<sea_orm::Select<entities::entries::Entity>> {
        let field = self.validate_field(fields, &filter.field_name, &[DataTypes::Object])?;

        query = query.inner_join(entities::entry_object_values::Entity)
            .filter(entities::entry_object_values::Column::FieldId.eq(field.id));

        match filter.comparison {
            ObjectComparison::HasProperty => {
                if let Some(property_path) = &filter.property_path {
                    // For now, return a helpful error message about object querying limitations
                    return Err(juniper::FieldError::new(
                        format!("Object property filtering for '{}' is not yet fully implemented. Consider using IsEmpty/IsNotEmpty for basic object filtering.", property_path),
                        Value::null(),
                    ));
                } else {
                    return Err(juniper::FieldError::new(
                        "property_path is required for HasProperty comparison".to_string(),
                        Value::null(),
                    ));
                }
            }
            ObjectComparison::PropertyEquals => {
                return Err(juniper::FieldError::new(
                    "PropertyEquals comparison not yet implemented. Consider using IsEmpty/IsNotEmpty for basic object filtering.".to_string(),
                    Value::null(),
                ));
            }
            ObjectComparison::PropertyContains => {
                return Err(juniper::FieldError::new(
                    "PropertyContains comparison not yet implemented. Consider using IsEmpty/IsNotEmpty for basic object filtering.".to_string(),
                    Value::null(),
                ));
            }
            ObjectComparison::IsEmpty => {
                // Check if the JSON object is null (basic check)
                query = query.filter(entities::entry_object_values::Column::Value.is_null());
            }
            ObjectComparison::IsNotEmpty => {
                // Check if the JSON object is not null (basic check)
                query = query.filter(entities::entry_object_values::Column::Value.is_not_null());
            }
        }

        Ok(query)
    }
}
