#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, Address, String, symbol_short};

// Structure to store loan details
#[contracttype]
#[derive(Clone)]
pub struct Loan {
    pub loan_id: u64,
    pub borrower: Address,
    pub lender: Address,
    pub amount: u64,
    pub interest_rate: u64, // in basis points (e.g., 500 = 5%)
    pub duration: u64, // in seconds
    pub start_time: u64,
    pub is_active: bool,
    pub is_repaid: bool,
}

// Counter for generating unique loan IDs
const LOAN_COUNT: Symbol = symbol_short!("L_COUNT");

// Mapping loan ID to Loan struct
#[contracttype]
pub enum LoanBook {
    Loan(u64)
}

#[contract]
pub struct P2PLendingContract;

#[contractimpl]
impl P2PLendingContract {
    
    // Function to create a loan request by borrower
    pub fn request_loan(
        env: Env,
        borrower: Address,
        amount: u64,
        interest_rate: u64,
        duration: u64
    ) -> u64 {
        // Verify borrower identity
        borrower.require_auth();
        
        // Get and increment loan counter
        let mut loan_count: u64 = env.storage().instance().get(&LOAN_COUNT).unwrap_or(0);
        loan_count += 1;
        
        let _current_time = env.ledger().timestamp();
        
        // Create new loan request
        let new_loan = Loan {
            loan_id: loan_count,
            borrower: borrower.clone(),
            lender: Address::from_string(&String::from_str(&env, "placeholder")),
            amount,
            interest_rate,
            duration,
            start_time: 0,
            is_active: false,
            is_repaid: false,
        };
        
        // Store loan in contract storage
        env.storage().instance().set(&LoanBook::Loan(loan_count), &new_loan);
        env.storage().instance().set(&LOAN_COUNT, &loan_count);
        
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Loan request created with ID: {}", loan_count);
        loan_count
    }
    
    // Function for lender to fund a loan
    pub fn fund_loan(env: Env, loan_id: u64, lender: Address) {
        lender.require_auth();
        
        let mut loan = Self::view_loan(env.clone(), loan_id);
        
        if loan.loan_id == 0 {
            log!(&env, "Loan does not exist");
            panic!("Loan does not exist");
        }
        
        if loan.is_active {
            log!(&env, "Loan already funded");
            panic!("Loan already funded");
        }
        
        loan.lender = lender.clone();
        loan.is_active = true;
        loan.start_time = env.ledger().timestamp();
        
        env.storage().instance().set(&LoanBook::Loan(loan_id), &loan);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Loan {} funded by lender", loan_id);
    }
    
    // Function for borrower to repay loan
    pub fn repay_loan(env: Env, loan_id: u64, borrower: Address) {
        borrower.require_auth();
        
        let mut loan = Self::view_loan(env.clone(), loan_id);
        
        if loan.loan_id == 0 {
            log!(&env, "Loan does not exist");
            panic!("Loan does not exist");
        }
        
        if !loan.is_active {
            log!(&env, "Loan is not active");
            panic!("Loan is not active");
        }
        
        if loan.is_repaid {
            log!(&env, "Loan already repaid");
            panic!("Loan already repaid");
        }
        
        if loan.borrower != borrower {
            log!(&env, "Only borrower can repay");
            panic!("Only borrower can repay");
        }
        
        loan.is_repaid = true;
        loan.is_active = false;
        
        env.storage().instance().set(&LoanBook::Loan(loan_id), &loan);
        env.storage().instance().extend_ttl(5000, 5000);
        
        log!(&env, "Loan {} repaid successfully", loan_id);
    }
    
    // Function to view loan details
    pub fn view_loan(env: Env, loan_id: u64) -> Loan {
        let key = LoanBook::Loan(loan_id);
        
        env.storage().instance().get(&key).unwrap_or(Loan {
            loan_id: 0,
            borrower: Address::from_string(&String::from_str(&env, "not_found")),
            lender: Address::from_string(&String::from_str(&env, "not_found")),
            amount: 0,
            interest_rate: 0,
            duration: 0,
            start_time: 0,
            is_active: false,
            is_repaid: false,
        })
    }
}
