use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub report_id: String,
    pub intent_id: String,
    pub exchange: String,
    pub symbol: String,
    pub order_status: OrderStatus,
    pub filled_qty: Option<String>,
    pub avg_fill_price: Option<String>,
    pub fees: Option<FeeDetail>,
    pub exchange_order_id: Option<String>,
    pub raw_status: Option<String>,
    pub reconcile_state: Option<ReconcileState>,
    pub ts_report: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    Created,
    RiskApproved,
    Sent,
    Acked,
    PartiallyFilled,
    Filled,
    CancelPending,
    Canceled,
    Rejected,
    Expired,
    Unknown,
    Reconciled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconcileState {
    Pending,
    Matched,
    Mismatched,
    Repaired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeDetail {
    pub commission: String,
    pub commission_asset: String,
}
