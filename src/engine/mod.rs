mod client;
#[cfg(test)]
mod test;
mod transaction;
mod utils;

pub use client::Client;
pub use transaction::Transaction;

use merx::{get_fixed, get_traits, new_asset, Asset, Credit, Debt};

get_traits!();

// Create a new asset called usd with 5 decimal digits precision and a maximum value of
// 90_000_000_000_000
new_asset!(usd, 4, 14_000_000_000_000);

pub type Usd = Asset<usd::Value>;
/// Values < 0
pub type DebtUsd = Debt<usd::Value>;
/// Values >= 0
pub type CreditUsd = Credit<usd::Value>;

pub fn zero_usd() -> Usd {
    Usd::try_from(0).unwrap()
}

pub fn zero_usd_as_credit() -> CreditUsd {
    match Usd::try_from(0).unwrap() {
        merx::Asset::Credit(x) => x,
        _ => panic!("impossible"),
    }
}

pub fn engine(db: &mut crate::db::Db, transaction: Transaction, client_id: u16) {
    let client = db.get_client(client_id);
    match client {
        None => {
            let mut client = client::Client::new();
            client.handle_transaction(transaction);
            db.add_client(client_id, client);
        }
        Some(client) => {
            client.handle_transaction(transaction);
        }
    }
}
