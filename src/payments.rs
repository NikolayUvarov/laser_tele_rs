//! Payments: <https://core.telegram.org/bots/payments> and <https://core.telegram.org/bots/payments-stars>.
//! The flow: `send_invoice` -> (shipping query -> `answer_shipping_query`, only for `is_flexible` invoices) ->
//! pre-checkout query -> `answer_pre_checkout_query` within 10 seconds -> a message with `successful_payment` ->
//! deliver the goods

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::*;

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_zero(value: &i64) -> bool {
    *value == 0
}

/// An invoice sent by `send_invoice` ([`Message::invoice`])
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Invoice {
    pub title: String,
    pub description: String,
    pub start_parameter: String,
    pub currency: String,
    /// In the smallest units of the currency (cents...)
    pub total_amount: i64,
}

/// The address entered by the user
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShippingAddress {
    pub country_code: String,
    pub state: String,
    pub city: String,
    pub street_line1: String,
    pub street_line2: String,
    pub post_code: String,
}

/// The information about the order entered by the user
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OrderInfo {
    pub name: String,
    pub phone_number: String,
    pub email: String,
    pub shipping_address: Option<ShippingAddress>,
}

/// The information about a successful payment ([`Message::successful_payment`])
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SuccessfulPayment {
    pub currency: String,
    pub total_amount: i64,
    pub invoice_payload: String,
    pub subscription_expiration_date: i64,
    pub is_recurring: bool,
    pub is_first_recurring: bool,
    pub shipping_option_id: String,
    pub order_info: Option<OrderInfo>,
    /// Pass it to `refund_star_payment`
    pub telegram_payment_charge_id: String,
    pub provider_payment_charge_id: String,
}

/// The information about a refunded payment ([`Message::refunded_payment`])
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RefundedPayment {
    pub currency: String,
    pub total_amount: i64,
    pub invoice_payload: String,
    pub telegram_payment_charge_id: String,
    pub provider_payment_charge_id: String,
}

/// Sent for invoices with `is_flexible`, answer it with `answer_shipping_query`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShippingQuery {
    pub id: String,
    pub from: User,
    pub invoice_payload: String,
    pub shipping_address: ShippingAddress,
}

/// Sent before the payment, answer it with `answer_pre_checkout_query` within 10 seconds
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PreCheckoutQuery {
    pub id: String,
    pub from: User,
    pub currency: String,
    pub total_amount: i64,
    pub invoice_payload: String,
    pub shipping_option_id: String,
    pub order_info: Option<OrderInfo>,
}

/// Sent when a user bought paid media with a payload sent by the bot
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PaidMediaPurchased {
    pub from: User,
    pub paid_media_payload: String,
}

/// Sent when a payment subscription of a user is changed
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BotSubscriptionUpdated {
    pub user: User,
    pub invoice_payload: String,
    /// "canceled", "active" or "failed"
    pub state: String,
}

/// A part of the price: the goods, the delivery, the tax
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LabeledPrice {
    pub label: String,
    /// In the smallest units of the currency (cents...), in Stars for "XTR"
    pub amount: i64,
}

impl LabeledPrice {
    pub fn new(label: impl Into<String>, amount: i64) -> Self {
        LabeledPrice {
            label: label.into(),
            amount,
        }
    }
}

/// A way of delivery for `answer_shipping_query`
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShippingOption {
    pub id: String,
    pub title: String,
    pub prices: Vec<LabeledPrice>,
}

/// An invoice for `send_invoice` and `create_invoice_link`.
/// For payments in Telegram Stars use currency "XTR", an empty `provider_token` and exactly one price
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InvoiceConfig {
    /// 1-32 characters
    pub title: String,
    /// 1-255 characters
    pub description: String,
    /// Returned in the pre-checkout query and the successful payment, it is not shown to the user
    pub payload: String,
    /// From @BotFather, empty for Telegram Stars
    #[serde(skip_serializing_if = "String::is_empty")]
    pub provider_token: String,
    /// "XTR" for Telegram Stars, "USD", "EUR", "RUB"...
    pub currency: String,
    pub prices: Vec<LabeledPrice>,
    /// Only `create_invoice_link`, Stars: 2592000 (30 days) for a monthly subscription
    #[serde(skip_serializing_if = "is_zero")]
    pub subscription_period: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub max_tip_amount: i64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggested_tip_amounts: Vec<i64>,
    /// Only `send_invoice`: forwarded invoices open the bot with `/start <start_parameter>`
    #[serde(skip_serializing_if = "String::is_empty")]
    pub start_parameter: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub provider_data: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub photo_url: String,
    #[serde(skip_serializing_if = "is_zero")]
    pub photo_size: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub photo_width: i64,
    #[serde(skip_serializing_if = "is_zero")]
    pub photo_height: i64,
    #[serde(skip_serializing_if = "is_false")]
    pub need_name: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub need_phone_number: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub need_email: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub need_shipping_address: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub send_phone_number_to_provider: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub send_email_to_provider: bool,
    /// The price depends on the delivery, a shipping query is sent
    #[serde(skip_serializing_if = "is_false")]
    pub is_flexible: bool,
    /// Only `send_invoice`: the first button must be the Pay button, see [`Button::pay`]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_markup: Option<InlineKeyboard>,
}

impl Bot {
    /// Sends an invoice and returns the sent message
    pub async fn send_invoice(&self, chat_id: i64, invoice: &InvoiceConfig) -> Result<Message> {
        let mut params = serde_json::to_value(invoice)?;
        if let Value::Object(fields) = &mut params {
            fields.remove("subscription_period");
            fields.insert("chat_id".into(), json!(chat_id));
        }
        self.call_into("payments", "sendInvoice", &params).await
    }

    /// Creates a link to pay the invoice, it can be sent anywhere
    pub async fn create_invoice_link(&self, invoice: &InvoiceConfig) -> Result<String> {
        let mut params = serde_json::to_value(invoice)?;
        if let Value::Object(fields) = &mut params {
            fields.remove("start_parameter");
            fields.remove("reply_markup");
        }
        self.call_into("payments", "createInvoiceLink", &params)
            .await
    }

    /// Answers the shipping query with ways of delivery.
    /// A non-empty `error_message` (shown to the user) means that the delivery to the address is impossible
    pub async fn answer_shipping_query(
        &self,
        shipping_query_id: &str,
        options: &[ShippingOption],
        error_message: &str,
    ) -> Result<()> {
        let params = if error_message.is_empty() {
            json!({ "shipping_query_id": shipping_query_id, "ok": true, "shipping_options": options })
        } else {
            json!({ "shipping_query_id": shipping_query_id, "ok": false, "error_message": error_message })
        };
        self.call_json("payments", "answerShippingQuery", &params)
            .await
            .map(drop)
    }

    /// Confirms the payment (empty `error_message`) or cancels it with `error_message` shown to the user.
    /// It must be called within 10 seconds after the query is received
    pub async fn answer_pre_checkout_query(
        &self,
        pre_checkout_query_id: &str,
        error_message: &str,
    ) -> Result<()> {
        let params = if error_message.is_empty() {
            json!({ "pre_checkout_query_id": pre_checkout_query_id, "ok": true })
        } else {
            json!({ "pre_checkout_query_id": pre_checkout_query_id, "ok": false, "error_message": error_message })
        };
        self.call_json("payments", "answerPreCheckoutQuery", &params)
            .await
            .map(drop)
    }

    /// Returns Telegram Stars of the payment ([`SuccessfulPayment::telegram_payment_charge_id`]) to the user
    pub async fn refund_star_payment(
        &self,
        user_id: i64,
        telegram_payment_charge_id: &str,
    ) -> Result<()> {
        let params =
            json!({ "user_id": user_id, "telegram_payment_charge_id": telegram_payment_charge_id });
        self.call_json("payments", "refundStarPayment", &params)
            .await
            .map(drop)
    }

    /// Cancels (`is_canceled` true) or re-enables the subscription of the user paid in Telegram Stars
    pub async fn edit_user_star_subscription(
        &self,
        user_id: i64,
        telegram_payment_charge_id: &str,
        is_canceled: bool,
    ) -> Result<()> {
        let params = json!({
            "user_id": user_id,
            "telegram_payment_charge_id": telegram_payment_charge_id,
            "is_canceled": is_canceled,
        });
        self.call_json("payments", "editUserStarSubscription", &params)
            .await
            .map(drop)
    }
}
