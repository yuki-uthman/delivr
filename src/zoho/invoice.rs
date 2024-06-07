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
    pub customer_name: String,
    pub date: String,
    pub invoice_id: String,
    pub line_items: Vec<LineItem>,
    pub salesperson_name: String,
}

impl From<serde_json::Value> for Invoice {
    fn from(value: serde_json::Value) -> Self {
        serde_json::from_value(value).unwrap()
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct LineItem {
    pub name: String,
    pub rate: f64,
    pub quantity: f64,
    pub purchase_rate: f64,
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
}
