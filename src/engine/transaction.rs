use super::CreditUsd;
use super::DebtUsd;
use super::Usd;

#[derive(Debug, Clone)]
pub enum Transaction {
    Deposit(CreditUsd),
    Withdrawal(DebtUsd),
    Dispute(DebtUsd),
    Resolve(CreditUsd),
    Chargeback(DebtUsd),
}

impl Transaction {
    pub fn new_deposit(amount: CreditUsd) -> Self {
        Self::Deposit(amount)
    }

    pub fn new_withdrawl(amount: DebtUsd) -> Self {
        Self::Withdrawal(amount)
    }

    pub fn new_dispute(amount: DebtUsd) -> Self {
        Self::Dispute(amount)
    }

    pub fn new_resolve(amount: CreditUsd) -> Self {
        Self::Resolve(amount)
    }

    pub fn new_chargeback(amount: DebtUsd) -> Self {
        Self::Chargeback(amount)
    }

    pub fn get_amount(&self) -> Usd {
        match &self {
            Self::Deposit(x) => Usd::from(merx::Asset::Credit(x.clone())),
            Self::Withdrawal(x) => Usd::from(merx::Asset::Debt(x.clone())),
            Self::Dispute(x) => Usd::from(merx::Asset::Debt(x.clone())),
            Self::Resolve(x) => Usd::from(merx::Asset::Credit(x.clone())),
            Self::Chargeback(x) => Usd::from(merx::Asset::Debt(x.clone())),
        }
    }
}
