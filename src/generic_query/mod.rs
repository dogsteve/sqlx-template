use syn::Data;

pub enum QueryOperator {
    Equal,
    NotEqual,
    In,
    LessThan,
    GreaterThan,
    Like,
}

pub enum LogicalOperator {
    And,
    Or,
}

pub enum DataType {
    String,
    Number,
}
pub struct QueryObject {
    key: String,
    operator: QueryOperator,
    data: Vec<String>,
    data_type: Option<DataType>,
    logical_operator: Option<LogicalOperator>,
}

pub mod query;
