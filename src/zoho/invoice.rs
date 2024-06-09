use serde::{ser::{SerializeStruct, Serializer}, Deserialize};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct InvoiceIDs {
    #[serde(rename = "invoices")]
    pub inner: Vec<InvoiceID>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct InvoiceID {
    #[serde(rename = "invoice_id")]
    pub id: String,
}

impl From<serde_json::Value> for InvoiceIDs {
    fn from(value: serde_json::Value) -> Self {
        serde_json::from_value(value).unwrap()
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Invoices {
    pub invoices: Vec<Invoice>,
}

impl From<serde_json::Value> for Invoices {
    fn from(value: serde_json::Value) -> Self {
        serde_json::from_value(value).unwrap()
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Invoice {
    #[serde(deserialize_with = "de_deserialize")]
    pub created_time: chrono::NaiveDateTime,
    pub customer_name: String,
    pub date: String,
    pub invoice_id: String,
    pub line_items: Vec<LineItem>,
    pub salesperson_name: String,
    pub total: f64,
}

// 2024-05-27T19:26:32+0800
fn de_deserialize<'de, D>(deserializer: D) -> Result<chrono::NaiveDateTime, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let datetime = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%z")
        .map_err(serde::de::Error::custom)?;
    Ok(datetime)
}

impl From<serde_json::Value> for Invoice {
    fn from(value: serde_json::Value) -> Self {
        serde_json::from_value(value).unwrap()
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct LineItem {
    pub name: String,
    pub rate: f64,
    pub quantity: f64,
    pub purchase_rate: f64,
    pub item_total: f64,
}

impl LineItem {
    pub fn profit(&self) -> f64 {
        self.item_total - self.purchase_rate * self.quantity
    }
}

// Implement custom serialization to include `profit`
impl serde::Serialize for LineItem {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("LineItem", 6)?;
        state.serialize_field("item_profit", &self.profit())?;
        state.serialize_field("item_total", &self.item_total)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("purchase_rate", &self.purchase_rate)?;
        state.serialize_field("quantity", &self.quantity)?;
        state.serialize_field("rate", &self.rate)?;
        state.end()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::error::Result;

    #[test]
    fn invoice_from_json() -> Result<()> {
        // Read the file content as a string
        let data = std::fs::read_to_string("tests/invoice_response.txt")?;

        // Parse the string into a serde_json::Value
        let json_value: serde_json::Value = serde_json::from_str(&data)?;

        let invoice = json_value
            .get("invoice")
            .ok_or("invoice not found")
            .unwrap();
        let _ = Invoice::from(invoice.clone());

        Ok(())
    }

    #[test]
    fn line_item_serialize() -> Result<()> {
        let line_item = LineItem {
            name: "name".to_string(),
            rate: 11.0,
            quantity: 10.0,
            purchase_rate: 10.0,
            item_total: 110.0,
        };

        let serialized = serde_json::to_string(&line_item)?;

        assert_eq!(
            serialized,
            r#"{"item_profit":10.0,"item_total":110.0,"name":"name","purchase_rate":10.0,"quantity":10.0,"rate":11.0}"#
        );
        Ok(())
    }
}
