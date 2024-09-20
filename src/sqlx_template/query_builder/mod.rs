mod postgres;

use proc_macro2::Span;
use std::cmp::PartialEq;
use std::collections::HashMap;
use std::iter::Map;

pub trait EnumToStr {
    fn as_str(&self) -> &'static str;
}

#[derive(Clone)]
pub enum QueryOperator {
    Equal,
    NotEqual,
    In,
    NotIn,
    Like,
    NotLike,
}

impl EnumToStr for QueryOperator {
    fn as_str(&self) -> &'static str {
        match self {
            QueryOperator::Equal => "=",
            QueryOperator::NotEqual => "<>",
            QueryOperator::In => "IN",
            QueryOperator::NotIn => "NOT IN",
            QueryOperator::Like => "LIKE",
            QueryOperator::NotLike => "NOT LIKE"
        }
    }
}

#[derive(Clone)]
pub enum LogicOperator {
    Not,
    And,
    Or,
}

impl EnumToStr for LogicOperator {
    fn as_str(&self) -> &'static str {
        match self {
            LogicOperator::Not => "NOT",
            LogicOperator::And => "AND",
            LogicOperator::Or => "OR"
        }
    }
}

#[derive(Clone)]
pub struct WhereCondition {
    // Field name of string you want to query
    pub field_name: String,
    // Value of where condition, it could be one or more
    pub value: Vec<String>,
    // Logic query operator
    pub logic_operator: Option<LogicOperator>,
    // Logic operator
    pub operator: QueryOperator,
}

#[derive(Clone)]
pub enum QueryAction {
    Select,
    Update,
    Delete,
}

impl EnumToStr for QueryAction {
    fn as_str(&self) -> &'static str {
        match self {
            QueryAction::Select => "SELECT",
            QueryAction::Update => "UPDATE",
            QueryAction::Delete => "DELETE"
        }
    }
}

#[derive(Clone)]
pub struct QueryComponent {
    pub action: QueryAction,
    pub table_name: String,
    pub field_names: Vec<String>,
    pub operated_in: Option<Box<QueryComponent>>,
    pub params: Option<HashMap<String, String>>,
    pub where_conditions: Option<Vec<WhereCondition>>,
}

// Helper trait to interact data
pub trait QueryBuilder<T> {
    fn select_field(self, param: Vec<String>);
    fn from(self, builder: dyn QueryBuilder<T>);
    fn where_condition(self, param: Vec<WhereCondition>);
    fn update(self, param: Map<String, String>);
    fn delete(self);
}

impl QueryComponent {
    fn build_to_string(self) -> syn::Result<String> {
        let mut result = String::new();
        let action = self.action;
        let action_str = action.as_str();
        result.push_str(action_str);
        match action {
            QueryAction::Select => {
                if self.field_names.len() == 0 {
                    result.push_str(" * ")
                } else {
                    result.push_str(" ( ");
                    for field in self.field_names {
                        result.push_str(format!("{},", field).as_str());
                    }
                    result.remove(result.len() - 1);
                    result.push_str(" )");
                }
                result.push_str(" FROM ");
                if let Some(mut operated_in) = self.operated_in {
                    let operated_in = operated_in;
                    match operated_in.build_to_string() {
                        Ok(rs) => {
                            result.push_str("( ");
                            result.push_str(rs.as_str());
                            result.push_str(" )");
                        }
                        Err(e) => {
                            return Err(syn::Error::new(Span::call_site(), "Error when creating select query"));
                        }
                    }
                } else {
                    result.push_str(format!(" {} ", self.table_name).as_str())
                }
                if let Some(where_conditions) = self.where_conditions {
                    result.push_str(build_where_condition(&where_conditions).as_str());
                }
            }
            QueryAction::Update => {
                result.push_str(format!(" {} SET ", self.table_name).as_str());
                if let Some(params) = self.params {
                    for (k, v) in params.into_iter() {
                        result.push_str(format!(" {} = '{}', ", k, v).as_str());
                    }
                    // Remove last comma
                    result.remove(result.len() - 1);
                } else {
                    return Err(syn::Error::new(Span::call_site(), "Missing update param"));
                }
            }
            QueryAction::Delete => {
                result.push_str(format!(" FROM {} ", self.table_name).as_str());
                if let Some(where_conditions) = self.where_conditions {
                    result.push_str(build_where_condition(&where_conditions).as_str());
                } else {
                    return Err(syn::Error::new(Span::call_site(), "Missing delete condition"));
                }
            }
        }
        Ok(result)
    }
}

fn build_like_query(value: &[String]) -> String {
    let mut query_data = String::new();
    query_data.push_str("( ");
    let data_list = value.into_iter().map(
        |e| format!("'{}'", e)
    ).for_each(
        |e| {
            query_data.push_str(format!("{},", e).as_str())
        }
    );
    // Remove last comma
    query_data.remove(query_data.len() - 1);
    query_data.push_str(" )");
    query_data
}

fn build_where_condition(where_conditions: &Vec<WhereCondition>) -> String {
    let mut result = String::new();
    result.push_str(" WHERE ");
    for condition in where_conditions {
        let field_name = condition.field_name.as_str();
        let value = &condition.value;
        let mut query_data = String::new();

        match condition.operator {
            QueryOperator::Equal => {
                query_data.push_str(format!("{} = '{}' ", field_name, value[0]).as_str());
            }
            QueryOperator::NotEqual => {
                query_data.push_str(format!("{} <> '{}' ", field_name, value[0]).as_str());
            }
            QueryOperator::In => {
                query_data.push_str(format!("{} IN {} ", field_name, build_like_query(&value)).as_str());
            }
            QueryOperator::NotIn => {
                query_data.push_str(format!("{} NOT IN {} ", field_name, build_like_query(&value)).as_str());
            }
            QueryOperator::Like => {
                query_data.push_str(format!("{} LIKE '%{}%' ", field_name, value[0]).as_str());
            }
            QueryOperator::NotLike => {
                query_data.push_str(format!("{} NOT LIKE '%{}%' ", field_name, value[0]).as_str());
            }
        }
        if let Some(logical_operation) = &condition.logic_operator {
            result.push_str(format!("{} {} ", logical_operation.as_str(), query_data, ).as_str());
        } else {
            result.push_str(format!("{} ", query_data, ).as_str());
        }
    }
    result
}

#[test]
fn test_build_query() {
    let query = QueryComponent {
        action: QueryAction::Delete,
        table_name: String::from("TB"),
        field_names: vec![String::from("id")],
        operated_in: Some(Box::new(QueryComponent {
            action: QueryAction::Select,
            table_name: String::from("SUB_TB"),
            field_names: vec![],
            operated_in: None,
            params: None,
            where_conditions: Some(vec![WhereCondition {
                field_name: String::from("A"),
                value: vec![String::from("B")],
                logic_operator: None,
                operator: QueryOperator::In,
            }]),
        })),
        params: None,
        where_conditions: Some(vec![WhereCondition {
            field_name: String::from("A"),
            value: vec![String::from("B")],
            logic_operator: None,
            operator: QueryOperator::In,
        }]),
    };

    println!("{}", query.build_to_string().unwrap())
}