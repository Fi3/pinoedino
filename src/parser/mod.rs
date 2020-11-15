use crate::db::Db;
use crate::engine::Usd;
use std::convert::{From, Into, TryFrom, TryInto};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct InputRow_ {
    #[serde(rename = "type")]
    type_: String,
    client: u16,
    tx: u32,
    amount: Option<String>,
}

struct InputRow {
    type_: String,
    client: u16,
    tx: u32,
    amount: Option<String>,
    linked_amount: Option<Usd>,
}

impl From<InputRow_> for InputRow {
    fn from(input_row: InputRow_) -> Self {
        InputRow {
            type_: input_row.type_,
            client: input_row.client,
            tx: input_row.tx,
            amount: input_row.amount,
            linked_amount: None,
        }
    }
}

pub fn parse(path: String, db: &mut Db) {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_path(path)
        .expect("invalid file");

    for result in rdr.deserialize() {
        let row: Result<InputRow_, csv::Error> = result;
        match row {
            Err(e) => eprintln!("WARNING: ingored input {}", e),
            Ok(row) => {
                let mut row: InputRow = row.into();
                if row.type_ == "dispute" || row.type_ == "resolve" || row.type_ == "chargeback" {
                    let client_id = row.client;
                    let linked_tx = db.get_tx(row.tx);
                    match linked_tx {
                        None => {
                            eprintln!("WARNING: ingored row invalid linked tx id: {}", row.tx);
                            continue;
                        }
                        Some(linked_tx) => {
                            if linked_tx.client_id != row.client {
                                eprintln!(
                                    "WARNING: ingored row invalid linked tx client id: {}",
                                    row.client
                                );
                                continue;
                            }
                            // TODO It assume that they always refer to a deposit
                            // that is always a Credit so a Debt is needed fo dispute and
                            // chargeback and a Credit is needed for resolve
                            if row.type_ == "resolve" {
                                row.linked_amount = Some(linked_tx.parsed_tx.get_amount())
                            } else {
                                row.linked_amount =
                                    Some((linked_tx.parsed_tx.get_amount() * -1).unwrap())
                            }
                        }
                    }
                    let parsed_tx: Result<crate::engine::Transaction, ()> = row.try_into();
                    match parsed_tx {
                        Err(_) => continue,
                        Ok(parsed_tx) => {
                            crate::engine::engine(db, parsed_tx, client_id);
                        }
                    }
                } else {
                    let client_id = row.client;
                    let tx_id = row.tx;
                    let parsed_tx: Result<crate::engine::Transaction, ()> = row.try_into();
                    match parsed_tx {
                        Err(_) => continue,
                        Ok(parsed_tx) => {
                            if db.has_id(tx_id) {
                                db.add_tx(tx_id, parsed_tx.clone(), client_id)
                            }
                            crate::engine::engine(db, parsed_tx, client_id);
                        }
                    }
                }
            }
        }
    }
}

// It save the id of the transactions that are referenced by special txs
pub fn pre_parse(path: String, db: &mut Db) {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .trim(csv::Trim::All)
        .from_path(path)
        .expect("invalid file");
    for result in rdr.deserialize() {
        let row: Result<InputRow_, csv::Error> = result;
        match row {
            Err(e) => eprintln!("WARNING: ingored input {}", e),
            Ok(row) => {
                if row.type_ == "dispute" || row.type_ == "resolve" || row.type_ == "chargeback" {
                    db.add_id(row.tx);
                }
            }
        }
    }
}

impl TryFrom<InputRow> for crate::engine::Transaction {
    type Error = ();

    fn try_from(row: InputRow) -> Result<Self, ()> {
        if row.type_ == "withdraw" {
            match row.amount {
                None => {
                    eprintln!("WARNING: ingored row no amount: {:#?}", row.amount);
                    Err(())
                }
                Some(amount) => {
                    let usd = Usd::try_from(&format!("{}{}", "-", amount)[..]);
                    match usd {
                        Err(_) => {
                            eprintln!("WARNING: ingored row invalid amount: {}", amount);
                            Err(())
                        }
                        Ok(usd) => match usd {
                            merx::Asset::Credit(_) => {
                                eprintln!("WARNING: ingored row negative amount: {}", amount);
                                Err(())
                            }
                            merx::Asset::Debt(usd) => {
                                Ok(crate::engine::Transaction::new_withdrawl(usd))
                            }
                        },
                    }
                }
            }
        } else if row.type_ == "deposit" {
            match row.amount {
                None => {
                    eprintln!("WARNING: ingored row no amount: {:#?}", row.amount);
                    Err(())
                }
                Some(amount) => {
                    let usd = Usd::try_from(&amount[..]);
                    match usd {
                        Err(_) => {
                            eprintln!("WARNING: ingored row invalid amount: {}", amount);
                            Err(())
                        }
                        Ok(usd) => match usd {
                            merx::Asset::Debt(_) => {
                                eprintln!("WARNING: ingored row positive amount: {}", amount);
                                Err(())
                            }
                            merx::Asset::Credit(usd) => {
                                Ok(crate::engine::Transaction::new_deposit(usd))
                            }
                        },
                    }
                }
            }
        } else if row.type_ == "dispute" {
            match row.linked_amount {
                None => {
                    eprintln!("WARNING: ingored row no amount: {:#?}", row.amount);
                    Err(())
                }
                Some(amount) => match amount {
                    merx::Asset::Credit(_) => {
                        eprintln!("WARNING: ingored row positive amount: {:#?}", amount);
                        Err(())
                    }
                    merx::Asset::Debt(usd) => Ok(crate::engine::Transaction::new_dispute(usd)),
                },
            }
        } else if row.type_ == "resolve" {
            match row.linked_amount {
                None => {
                    eprintln!("WARNING: ingored row no amount: {:#?}", row.amount);
                    Err(())
                }
                Some(amount) => match amount {
                    merx::Asset::Debt(_) => {
                        eprintln!("WARNING: ingored row negative amount: {:#?}", amount);
                        Err(())
                    }
                    merx::Asset::Credit(usd) => Ok(crate::engine::Transaction::new_resolve(usd)),
                },
            }
        } else if row.type_ == "chargeback" {
            match row.linked_amount {
                None => {
                    eprintln!("WARNING: ingored row no amount: {:#?}", row.amount);
                    Err(())
                }
                Some(amount) => match amount {
                    merx::Asset::Credit(_) => {
                        eprintln!("WARNING: ingored row positive amount: {:#?}", amount);
                        Err(())
                    }
                    merx::Asset::Debt(usd) => Ok(crate::engine::Transaction::new_chargeback(usd)),
                },
            }
        } else {
            eprintln!("WARNING: ignored row unknown type: {}", row.type_);
            Err(())
        }
    }
}
