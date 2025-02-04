// Copyright 2022 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// Borrow from apache/arrow/rust/datafusion/src/sql/sql_parser
// See notice.md

use sqlparser::ast::SetExpr;
use sqlparser::ast::Statement;
use sqlparser::ast::StreamValues;
use sqlparser::ast::Values;
use sqlparser::parser::ParserError;
use sqlparser::tokenizer::QueryOffset;

use crate::parser_err;
use crate::sql::statements::DfInsertStatement;
use crate::sql::statements::InsertSource;
use crate::sql::DfParser;
use crate::sql::DfStatement;

impl<'a> DfParser<'a> {
    pub(crate) fn parse_insert(&mut self) -> Result<DfStatement<'a>, ParserError> {
        self.parser.next_token();
        match self.parser.parse_stream_values_insert()? {
            Statement::Insert {
                or,
                table_name,
                columns,
                overwrite,
                source,
                partitioned,
                format,
                after_columns,
                table,
                on,
            } => {
                let insert_source = match source {
                    None => Ok(InsertSource::Empty),
                    Some(source) => match source.body {
                        SetExpr::Values(Values(_, stream_value)) => {
                            self.get_values_str(&stream_value).map(InsertSource::Values)
                        }
                        SetExpr::Select(_) => Ok(InsertSource::Select(source)),
                        _ => Err(ParserError::ParserError(
                            "Insert must be have values or select source.".to_string(),
                        )),
                    },
                }?;

                Ok(DfStatement::InsertQuery(DfInsertStatement {
                    or,
                    object_name: table_name,
                    columns,
                    overwrite,
                    source: insert_source,
                    partitioned,
                    format,
                    after_columns,
                    table,
                    on,
                }))
            }
            _ => parser_err!("Expect set insert statement"),
        }
    }

    fn get_values_str(&self, values_info: &StreamValues) -> Result<&'a str, ParserError> {
        let start = &values_info.start;
        let end = &values_info.end;
        let sql = self.sql;

        match (start, end) {
            (QueryOffset::Normal(start), QueryOffset::Normal(end)) => {
                let start = *start as usize;
                let end = *end as usize;

                Ok(&sql[start..end])
            }
            (QueryOffset::Normal(start), QueryOffset::EOF) => Ok(&sql[*start as usize..]),
            _ => parser_err!(format!(
                "Unexpected values position info, start:{}, end:{}",
                start, end,
            )),
        }
    }
}
