use super::transaction::Transaction;
use super::utils;
use super::CreditUsd;
use super::DebtUsd;
use super::FixedToInt;
use super::Usd;
use super::{zero_usd, zero_usd_as_credit};

lazy_static! {
    static ref ZERO: Usd = super::zero_usd();
    static ref C_ZERO: CreditUsd = super::zero_usd_as_credit();
}

/// Overflow used only for cashbacks if a tx try to withdrawl more than avaible it just fail
#[derive(Debug)]
pub struct Client {
    pub locked: bool,
    pub total: CreditUsd,
    pub held: Option<DebtUsd>,
    pub overflow: Option<DebtUsd>,
}

impl Client {
    pub fn new() -> Self {
        Client {
            locked: false,
            total: *C_ZERO,
            held: None,
            overflow: None,
        }
    }

    pub fn handle_transaction(&mut self, transaction: Transaction) {
        if self.locked {
            return ();
        }
        match transaction {
            Transaction::Deposit(usd) => match self.deposit(usd) {
                Some(()) => (),
                None => utils::print_warning(&transaction, &self),
            },
            Transaction::Withdrawal(usd) => match self.withdrawal(usd) {
                Some(()) => (),
                None => utils::print_warning(&transaction, &self),
            },
            Transaction::Dispute(usd) => match self.held(usd) {
                Some(()) => (),
                None => utils::print_warning(&transaction, &self),
            },
            Transaction::Resolve(usd) => match self.release(usd) {
                Some(()) => (),
                None => utils::print_warning(&transaction, &self),
            },
            Transaction::Chargeback(usd) => match self.chargeback(usd) {
                Some(()) => (),
                None => utils::print_warning(&transaction, &self),
            },
        }
    }

    pub fn avaiable_amount(&self) -> CreditUsd {
        match self.held {
            None => self.total,
            Some(held) => match (self.total - held).expect("impossible state") {
                merx::Asset::Debt(_) => panic!("impossible state"),
                merx::Asset::Credit(x) => x,
            },
        }
    }

    fn avaiable_amount_from_new(&self, new_total: CreditUsd) -> Usd {
        match self.held {
            None => merx::Asset::Credit(new_total),
            Some(held) => (new_total - held).expect("impossible state"),
        }
    }

    fn deposit(&mut self, usd: CreditUsd) -> Option<()> {
        let new_amount = (self.total + usd)?;
        self.total = new_amount;
        Some(())
    }

    fn withdrawal(&mut self, usd: DebtUsd) -> Option<()> {
        let new_total = (self.total - usd)?;

        // Check if required amount is bigger than total amount
        match new_total {
            merx::Asset::Debt(_) => None,
            merx::Asset::Credit(new_total) => {
                // Check if required amount is bigger than (total amount + held amount)
                match self.avaiable_amount_from_new(new_total) {
                    merx::Asset::Debt(_) => None,
                    merx::Asset::Credit(_) => {
                        self.total = new_total;
                        Some(())
                    }
                }
            }
        }
    }

    fn unwrap_held_or_0(&self) -> Usd {
        match self.held {
            None => zero_usd(),
            Some(held) => merx::Asset::Debt(held),
        }
    }

    fn held(&mut self, usd: DebtUsd) -> Option<()> {
        let new_held = (self.unwrap_held_or_0() + merx::Asset::Debt(usd))?;
        match new_held {
            merx::Asset::Credit(_) => panic!("impossible state"),
            merx::Asset::Debt(new_held) => {
                let avaiable = (self.total - new_held)?;
                match avaiable {
                    merx::Asset::Debt(_) => {
                        //self.overflow = Some(overflow);
                        self.held = Some(new_held);
                        Some(())
                    }
                    merx::Asset::Credit(_) => {
                        self.held = Some(new_held);
                        Some(())
                    }
                }
            }
        }
    }

    fn release(&mut self, usd: CreditUsd) -> Option<()> {
        match self.held {
            None => None,
            Some(held) => match (held + usd)? {
                merx::Asset::Debt(new_held) => {
                    self.held = Some(new_held);
                    Some(())
                }
                // If new held is a credit can be 0 and this is fine
                // If is bigger than 0 we are trying to release more found
                // thane the ones helded and this is must be an error
                merx::Asset::Credit(held) => {
                    if merx::Asset::Credit(held).to_int() == 0 {
                        self.held = None;
                        // If held is 0 overflow must be 0
                        // self.overflow = None;
                        Some(())
                    } else {
                        None
                    }
                }
            },
        }
    }

    fn update_total(&mut self, chargeback: DebtUsd) -> Option<()> {
        match (self.total - chargeback)? {
            merx::Asset::Debt(overflow) => {
                self.overflow = Some(overflow);
                self.total = zero_usd_as_credit();
                Some(())
            }
            merx::Asset::Credit(new_total) => {
                self.total = new_total;
                Some(())
            }
        }
    }

    fn chargeback(&mut self, usd: DebtUsd) -> Option<()> {
        let usd_to_release = (merx::Asset::Debt(usd) * -1).expect("impossible");
        self.locked = true;
        match usd_to_release {
            merx::Asset::Credit(usd_to_release) => {
                match self.release(usd_to_release) {
                    Some(_) => (),
                    None => {
                        eprintln!("WARNING: chargeback bigger than dispute");
                        self.held = None
                    }
                };
                self.update_total(usd);
                Some(())
            }
            _ => panic!("impossible"),
        }
    }
    #[cfg(test)]
    pub fn new_(
        locked: bool,
        total: CreditUsd,
        held: Option<DebtUsd>,
        overflow: Option<DebtUsd>,
    ) -> Self {
        Client {
            locked,
            total,
            held,
            overflow,
        }
    }
}
