use super::client::Client;
use super::transaction::Transaction;
use super::CreditUsd;
use super::Usd;
use quickcheck::{quickcheck, TestResult};

use std::convert::TryFrom;

const MAX: u128 = 90_000_000_000;
lazy_static! {
    static ref ZERO: Usd = super::zero_usd();
    static ref C_ZERO: CreditUsd = super::zero_usd_as_credit();
}

fn client_with_amount(total: Usd) -> Client {
    match total {
        merx::Asset::Debt(_) => panic!("impossible"),
        merx::Asset::Credit(total) => Client::new_(false, total, None, None),
    }
}

fn client_with_held(held: Usd) -> Client {
    let total: Usd = Usd::try_from(90).unwrap();
    match (total, held) {
        (merx::Asset::Credit(total), merx::Asset::Debt(held)) => {
            Client::new_(false, total, Some(held), None)
        }
        (_, _) => panic!("impossible"),
    }
}

fn client_with_held_and_total(held: Usd, total: Usd) -> Client {
    match (total, held) {
        (merx::Asset::Credit(total), merx::Asset::Debt(held)) => {
            Client::new_(false, total, Some(held), None)
        }
        (_, _) => panic!("impossible"),
    }
}

fn deposit_with_amount(amount: Usd) -> Transaction {
    match amount {
        merx::Asset::Debt(_) => panic!("impossible"),
        merx::Asset::Credit(amount) => Transaction::new_deposit(amount),
    }
}

fn withdrawl_with_amount(amount: Usd) -> Transaction {
    match amount {
        merx::Asset::Credit(_) => panic!("impossible"),
        merx::Asset::Debt(amount) => Transaction::new_withdrawl(amount),
    }
}

fn dispute_with_amount(amount: Usd) -> Transaction {
    match amount {
        merx::Asset::Credit(_) => panic!("impossible"),
        merx::Asset::Debt(amount) => Transaction::new_dispute(amount),
    }
}

fn resolve_with_amount(amount: Usd) -> Transaction {
    match amount {
        merx::Asset::Debt(_) => panic!("impossible"),
        merx::Asset::Credit(amount) => Transaction::new_resolve(amount),
    }
}

fn chargeback_with_amount(amount: Usd) -> Transaction {
    match amount {
        merx::Asset::Credit(_) => panic!("impossible"),
        merx::Asset::Debt(amount) => Transaction::new_chargeback(amount),
    }
}

#[quickcheck]
fn prop_deposit_increments_total(deposit: u128, total: u128) -> TestResult {
    if (deposit + total) > MAX {
        return TestResult::discard();
    }
    let deposit = Usd::try_from(deposit as i128);
    let total = Usd::try_from(total as i128);
    match (deposit, total) {
        (Ok(deposit), Ok(total)) => {
            let mut client = client_with_amount(total);
            let transaction = deposit_with_amount(deposit);
            client.handle_transaction(transaction);
            let client_total = Usd::from(merx::Asset::Credit(client.total));
            TestResult::from_bool(client_total == (deposit + total).unwrap())
        }
        _ => TestResult::discard(),
    }
}

#[quickcheck]
fn prop_withdrawal_decrements_total_if_possible(withdrawl: u128, total: u128) -> TestResult {
    if (withdrawl > MAX) || (total > MAX) || withdrawl == 0 {
        return TestResult::discard();
    }
    let possible_transaction = total >= withdrawl;
    let withdrawal = Usd::try_from(withdrawl as i128 * -1);
    let total = Usd::try_from(total as i128);
    match (withdrawal, total) {
        (Ok(withdrawal), Ok(total)) => {
            let mut client = client_with_amount(total);
            let transaction = withdrawl_with_amount(withdrawal);
            client.handle_transaction(transaction);
            let client_total = Usd::from(merx::Asset::Credit(client.total));
            if possible_transaction {
                TestResult::from_bool(client_total == (total + withdrawal).unwrap())
            } else {
                TestResult::from_bool(client_total == total)
            }
        }
        _ => TestResult::discard(),
    }
}

#[quickcheck]
fn prop_deposit_increments_handle(total: u128, held: u128) -> TestResult {
    if (total > MAX) || (held > MAX) || held == 0 {
        return TestResult::discard();
    }
    let held = Usd::try_from(held as i128 * -1);
    let total = Usd::try_from(total as i128);
    match (held, total) {
        (Ok(held), Ok(total)) => {
            let mut client = client_with_amount(total);
            let transaction = dispute_with_amount(held);
            client.handle_transaction(transaction);
            let client_held = Usd::from(merx::Asset::Debt(client.held.unwrap()));
            TestResult::from_bool(client_held == held)
        }
        _ => TestResult::discard(),
    }
}

#[quickcheck]
fn prop_resolve_decrements_held_if_possible(held: u128, release: u128) -> TestResult {
    if (held > MAX) || (release > MAX) || held == 0 {
        return TestResult::discard();
    }
    let possible_transaction = held > release;
    let same_amounts = held == release;
    let held = Usd::try_from(held as i128 * -1);
    let release = Usd::try_from(release as i128);
    match (held, release) {
        (Ok(held), Ok(release)) => {
            let mut client = client_with_held(held);
            let transaction = resolve_with_amount(release);
            client.handle_transaction(transaction);
            if possible_transaction {
                let client_held = Usd::from(merx::Asset::Debt(client.held.unwrap()));
                TestResult::from_bool(client_held == (held + release).unwrap())
            } else if same_amounts {
                TestResult::from_bool(client.held.is_none())
            } else {
                let client_held = Usd::from(merx::Asset::Debt(client.held.unwrap()));
                TestResult::from_bool(client_held == held)
            }
        }
        _ => TestResult::discard(),
    }
}

#[quickcheck]
fn prop_resolve_chargeback(held_: u128, total_: u128, chargeback_: u128) -> TestResult {
    // uncomment to test special condition TODO
    // let held_ = 4 as u128;
    // let total_ = 4 as u128;
    // let chargeback_ = 4 as u128;
    if (held_ > MAX) || (total_ > MAX) || held_ == 0 || chargeback_ > MAX || chargeback_ == 0 {
        return TestResult::discard();
    }

    let held = Usd::try_from(held_ as i128 * -1);
    let chargeback = Usd::try_from(chargeback_ as i128 * -1);
    let total = Usd::try_from(total_ as i128);

    match (held, total, chargeback) {
        (Ok(held), Ok(total), Ok(chargeback)) => {
            let mut client = client_with_held_and_total(held, total);
            let transaction = chargeback_with_amount(chargeback);
            client.handle_transaction(transaction);
            if chargeback_ > total_ && chargeback_ > held_ {
                let condition1 = client.total == *C_ZERO;
                let condition2 = client.held == None;
                let condition3 = Usd::from(merx::Asset::Debt(client.overflow.unwrap()))
                    == (total + chargeback).unwrap();
                TestResult::from_bool(condition1 && condition2 && condition3)
            } else if chargeback_ < total_ && chargeback_ < held_ {
                let condition1 =
                    Usd::from(merx::Asset::Credit(client.total)) == (total + chargeback).unwrap();
                let condition2 = Usd::from(merx::Asset::Debt(client.held.unwrap()))
                    == ((chargeback * -1).unwrap() + held).unwrap();
                let condition3 = client.overflow == None;
                TestResult::from_bool(condition1 && condition2 && condition3)
            } else if chargeback_ == total_ && chargeback_ == held_ {
                let condition1 = client.total == *C_ZERO;
                let condition2 = client.held == None;
                let condition3 = client.overflow == None;
                TestResult::from_bool(condition1 && condition2 && condition3)
            } else if chargeback_ == total_ && chargeback_ > held_ {
                let condition1 = client.total == *C_ZERO;
                let condition2 = client.held == None;
                let condition3 = client.overflow == None;
                TestResult::from_bool(condition1 && condition2 && condition3)
            } else if chargeback_ == total_ && chargeback_ < held_ {
                let condition1 = client.total == *C_ZERO;
                let condition2 = Usd::from(merx::Asset::Debt(client.held.unwrap()))
                    == ((chargeback * -1).unwrap() + held).unwrap();
                let condition3 = client.overflow == None;
                TestResult::from_bool(condition1 && condition2 && condition3)
            } else if chargeback_ > total_ && chargeback_ == held_ {
                let condition1 = client.total == *C_ZERO;
                let condition2 = client.held == None;
                let condition3 = Usd::from(merx::Asset::Debt(client.overflow.unwrap()))
                    == (total + chargeback).unwrap();
                TestResult::from_bool(condition1 && condition2 && condition3)
            } else if chargeback_ < total_ && chargeback_ == held_ {
                let condition1 =
                    Usd::from(merx::Asset::Credit(client.total)) == (total + chargeback).unwrap();
                let condition2 = client.held == None;
                let condition3 = client.overflow == None;
                TestResult::from_bool(condition1 && condition2 && condition3)
            } else if chargeback_ < total_ && chargeback_ > held_ {
                let condition1 =
                    Usd::from(merx::Asset::Credit(client.total)) == (total + chargeback).unwrap();
                let condition2 = client.held == None;
                let condition3 = client.overflow == None;
                TestResult::from_bool(condition1 && condition2 && condition3)
            } else if chargeback_ > total_ && chargeback_ < held_ {
                let condition1 = client.total == *C_ZERO;
                let condition2 = Usd::from(merx::Asset::Debt(client.held.unwrap()))
                    == ((chargeback * -1).unwrap() + held).unwrap();
                let condition3 = Usd::from(merx::Asset::Debt(client.overflow.unwrap()))
                    == (total + chargeback).unwrap();
                TestResult::from_bool(condition1 && condition2 && condition3)
            } else {
                panic!("forgot case")
            }
        }
        _ => TestResult::discard(),
    }
}
