use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Asset {
    pub symbol: String,
    pub name: String,
    pub current_price_cents: i64,
}

impl Asset {
    pub fn new(symbol: impl Into<String>, name: impl Into<String>, current_price_cents: i64) -> Self {
        Self {
            symbol: symbol.into().to_uppercase(),
            name: name.into(),
            current_price_cents,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Purchase {
    pub id: Uuid,
    pub user_id: Uuid,
    pub symbol: String,
    pub quantity: f64,
    pub paid_price_cents: i64,
    pub purchased_at: DateTime<Utc>,
}

impl Purchase {
    pub fn new(user_id: Uuid, symbol: impl Into<String>, quantity: f64, paid_price_cents: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            symbol: symbol.into().to_uppercase(),
            quantity,
            paid_price_cents,
            purchased_at: Utc::now(),
        }
    }

    pub fn invested_cents(&self) -> i64 {
        money_times_quantity(self.paid_price_cents, self.quantity)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
}

impl User {
    pub fn new(name: impl Into<String>, email: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            email: email.into().to_lowercase(),
            password: password.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub symbol: String,
    pub quantity: f64,
    pub invested_cents: i64,
    pub current_value_cents: i64,
    pub profit_cents: i64,
    pub purchases: Vec<Purchase>,
}

pub fn money_times_quantity(price_cents: i64, quantity: f64) -> i64 {
    ((price_cents as f64) * quantity).round() as i64
}

pub fn format_money(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let abs = cents.abs();
    format!("{}R$ {}.{:02}", sign, abs / 100, abs % 100)
}

pub fn build_positions(assets: &[Asset], purchases: &[Purchase], user_id: Uuid) -> Vec<Position> {
    let mut positions: Vec<Position> = Vec::new();

    for purchase in purchases.iter().filter(|purchase| purchase.user_id == user_id) {
        let current_price = assets
            .iter()
            .find(|asset| asset.symbol == purchase.symbol)
            .map(|asset| asset.current_price_cents)
            .unwrap_or(purchase.paid_price_cents);

        if let Some(position) = positions.iter_mut().find(|position| position.symbol == purchase.symbol) {
            position.quantity += purchase.quantity;
            position.invested_cents += purchase.invested_cents();
            position.current_value_cents += money_times_quantity(current_price, purchase.quantity);
            position.profit_cents = position.current_value_cents - position.invested_cents;
            position.purchases.push(purchase.clone());
        } else {
            let invested = purchase.invested_cents();
            let current_value = money_times_quantity(current_price, purchase.quantity);
            positions.push(Position {
                symbol: purchase.symbol.clone(),
                quantity: purchase.quantity,
                invested_cents: invested,
                current_value_cents: current_value,
                profit_cents: current_value - invested,
                purchases: vec![purchase.clone()],
            });
        }
    }

    positions.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_money_in_brazilian_reais() {
        assert_eq!(format_money(123456), "R$ 1234.56");
        assert_eq!(format_money(-501), "-R$ 5.01");
    }

    #[test]
    fn calculates_positions_grouping_purchases_by_asset() {
        let user = User::new("Rafael", "rafael@example.com", "123456");
        let assets = vec![Asset::new("btc", "Bitcoin", 1000), Asset::new("usd", "Dólar", 520)];
        let purchases = vec![
            Purchase::new(user.id, "btc", 2.0, 700),
            Purchase::new(user.id, "btc", 1.0, 1200),
            Purchase::new(user.id, "usd", 10.0, 500),
        ];

        let positions = build_positions(&assets, &purchases, user.id);

        assert_eq!(positions.len(), 2);
        assert_eq!(positions[0].symbol, "BTC");
        assert_eq!(positions[0].quantity, 3.0);
        assert_eq!(positions[0].invested_cents, 2600);
        assert_eq!(positions[0].current_value_cents, 3000);
        assert_eq!(positions[0].profit_cents, 400);
    }
}
