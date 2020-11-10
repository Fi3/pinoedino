use crate::engine::Client;
use crate::engine::{CreditUsd, DebtUsd, Usd};
use merx::fixed::IsFixed;
use std::convert::{From, Into};

struct Fixed(i128, i128, u128);

impl Fixed {
    pub fn format(&self) -> String {
        format!("{}.{}", (self.0.abs()), (self.1.abs() / 10))
    }
}

impl From<Usd> for Fixed {
    fn from(usd: Usd) -> Fixed {
        let (a, b, c) = usd.get_inner().to_parts();
        Fixed(a, b, c)
    }
}
impl From<DebtUsd> for Fixed {
    fn from(usd: DebtUsd) -> Fixed {
        let (a, b, c) = Usd::from(merx::Asset::Debt(usd)).get_inner().to_parts();
        Fixed(a, b, c)
    }
}
impl From<CreditUsd> for Fixed {
    fn from(usd: CreditUsd) -> Fixed {
        let (a, b, c) = Usd::from(merx::Asset::Credit(usd)).get_inner().to_parts();
        Fixed(a, b, c)
    }
}

pub struct OutputRow {
    available: Fixed,
    held: Fixed,
    total: Fixed,
    locked: bool,
}

impl OutputRow {
    pub fn print_header() {
        println!("client,available,held,total,locked");
    }

    pub fn print(&self, id: &u16) {
        println!(
            "{},{},{},{},{}",
            id,
            self.available.format(),
            self.held.format(),
            self.total.format(),
            self.locked
        );
    }
}

impl From<&Client> for OutputRow {
    fn from(client: &Client) -> Self {
        let available: Fixed = client.avaiable_amount().into();
        let held: Fixed = match client.held {
            None => Fixed(0, 0, 0),
            Some(held) => held.into(),
        };
        let total: Fixed = match client.overflow {
            None => client.total.into(),
            Some(held) => held.into(),
        };
        OutputRow {
            available,
            held,
            total,
            locked: client.locked,
        }
    }
}
