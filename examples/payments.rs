//! Shop bot selling for Telegram Stars: /buy sends an invoice, /link sends a link to pay,
//! /refund returns the last payment. Payments in Stars (currency "XTR") need no payment provider.
//!
//! Run: TG_API_KEY=<token> cargo run --example payments

use std::collections::HashMap;

use laser_tele::{Config, InvoiceConfig, LabeledPrice, Message, UpdateKind, blocking};

fn coffee() -> InvoiceConfig {
    InvoiceConfig {
        title: "Coffee".into(),
        description: "A big cup of hot coffee".into(),
        // returned in the pre-checkout query and the successful payment
        payload: "coffee".into(),
        currency: "XTR".into(),
        prices: vec![LabeledPrice::new("Coffee", 1)],
        photo_url: "https://telegram.org/img/t_logo.png".into(),
        ..Default::default()
    }
}

fn main() -> laser_tele::Result<()> {
    let bot = blocking::Bot::new(Config::default())?;
    // the last payment of every user: user ID -> telegram_payment_charge_id
    let mut payments: HashMap<i64, String> = HashMap::new();

    bot.run(|update| {
        let result = match &update.kind {
            UpdateKind::Message(message) => on_message(&bot, &mut payments, message),
            UpdateKind::PreCheckoutQuery(query) => {
                // the last check before the payment, it must be answered within 10 seconds:
                // "" confirms the payment, a text cancels it and is shown to the user
                if query.invoice_payload == "coffee" {
                    bot.answer_pre_checkout_query(&query.id, "")
                } else {
                    bot.answer_pre_checkout_query(&query.id, "Sorry, this product is sold out")
                }
            }
            _ => Ok(()),
        };
        if let Err(e) = result {
            eprintln!("Error: {e}");
        }
    });
    Ok(())
}

fn on_message(
    bot: &blocking::Bot,
    payments: &mut HashMap<i64, String>,
    message: &Message,
) -> laser_tele::Result<()> {
    let chat_id = message.chat.id;
    let user_id = message.from.as_ref().map(|user| user.id).unwrap_or(0);

    // the payment is done, deliver the goods
    if let Some(payment) = &message.successful_payment {
        payments.insert(user_id, payment.telegram_payment_charge_id.clone());
        bot.send_message(
            chat_id,
            &format!(
                "Thank you! You paid {} ⭐. Here is your coffee ☕",
                payment.total_amount
            ),
        )?;
        return Ok(());
    }

    match message.text.as_str() {
        "/buy" => {
            bot.send_invoice(chat_id, &coffee())?;
        }
        "/link" => {
            // the link can be sent anywhere, e.g. to a website
            let link = bot.create_invoice_link(&coffee())?;
            bot.send_message(chat_id, &format!("Pay here: {link}"))?;
        }
        "/refund" => match payments.remove(&user_id) {
            Some(charge_id) => {
                bot.refund_star_payment(user_id, &charge_id)?;
                bot.send_message(chat_id, "Your Stars are returned")?;
            }
            None => {
                bot.send_message(chat_id, "You have no payments to refund")?;
            }
        },
        _ => {}
    }
    Ok(())
}
