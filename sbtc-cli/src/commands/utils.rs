use serde::Serialize;

#[derive(Serialize)]
pub struct TransactionData {
    pub id: String,
    pub hex: String,
}
