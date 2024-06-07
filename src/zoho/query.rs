use chrono::NaiveDate;
use serde::Serialize;

use crate::error::{Error, Result};

#[derive(Serialize, Debug, Clone)]
pub struct Query<'a> {
    pub organization_id: &'a str,
    pub date: Option<NaiveDate>,
}

#[derive(Default)]
pub struct QueryBuilder<'a> {
    organization_id: Option<&'a str>,
    date: Option<NaiveDate>,
}

impl<'a> Query<'a> {
    pub fn builder() -> QueryBuilder<'a> {
        QueryBuilder::default()
    }
}

impl<'a> QueryBuilder<'a> {
    pub fn organization_id(mut self, organization_id: &'a str) -> Self {
        self.organization_id = Some(organization_id);
        self
    }

    pub fn date(mut self, date: &str) -> Result<Self> {
        let date = NaiveDate::parse_from_str(date, "%Y-%m-%d")?;
        self.date = Some(date);
        Ok(self)
    }

    pub fn build(self) -> Result<Query<'a>> {
        if let Some(organization_id) = self.organization_id {
            Ok(Query {
                organization_id,
                date: self.date,
            })
        } else {
            Err(Error::custom("Missing organization_id"))
        }
    }
}
