
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};

fn main() -> Result<(), std::io::Error> {
    // First we try to read in any existing accounts information from file.
    let mut accounts = get_existing_accounts_data_from_file();

    // We find the path to the transactions.csv file in args[1]
    let args: Vec<String> = std:: env::args().collect();
    let transaction_path = &args[1];

    // Set up a BufReader to read in transactions.csv file bit by bit
    let file = std::fs::File::open(transaction_path)?;
    let reader = BufReader::new(file);

    // We need to track of processed transactions inorder to check old amounts for
    // disputes, resolves, and chargebacks
    let mut transaction_records: Vec<Transaction> = vec![];

    for line in reader.lines() {
        // We do not process unreadable lines.
        if let Ok(l) = line {
            if let Some(transaction) = Transaction::new(l) {
                process_transaction(&transaction, &mut accounts, &transaction_records);
                transaction_records.push(transaction); // Keep track of processed transactions
            }
        }
    }

    // Time to serialize accounts and write to stdout!
    let mut writer = csv::Writer::from_writer(std::io::stdout());
    for mut a in accounts {
        round_to_decimals(&mut a.available);
        round_to_decimals(&mut a.held);
        round_to_decimals(&mut a.total);
        writer.serialize(a)?;
    }

    writer.flush()?;

    Ok(())
}

/// Quick and dirty hard-coded function rouding off f32's to 4 decimal places
fn round_to_decimals(x: &mut f32) -> f32 {
    format!("{:.4}", x.to_string()).parse().unwrap()
}

/// This function is responsible for processing all the transactions and making updates to the accounts
fn process_transaction(
    transaction: &Transaction,
    accounts: &mut Vec<Account>,
    transactions: &Vec<Transaction>,
) {
    // Fetch the account from client_id in transaction
    let account = match accounts.iter_mut().find(|a| a.client == transaction.client) {
        Some(a) => a,
        None => {
            accounts.push(Account::new(transaction.client));
            accounts.last_mut().unwrap() // Safe to assume it exists!
        }
    };

    // Match against transaction in order to determine how to update the account 
    match transaction {
        Transaction {
            client: _,
            tx: _,
            tx_type: TransactionType::Deposit(amount),
        } => {
            deposit(account, *amount);
        }
        Transaction {
            client: _,
            tx: _,
            tx_type: TransactionType::WithDrawal(amount),
        } => withdraw(account, *amount),
        Transaction {
            client: _,
            tx,
            tx_type: TransactionType::Dispute,
        } => {
            // We ignore cases where the transaction was not found
            if let Some(amount) = get_amount_from_tx_id(transactions, *tx) {
                dispute(account, amount);
            }
        }
        Transaction {
            client: _,
            tx,
            tx_type: TransactionType::Resolve,
        } => {
            // We ignore cases where the transaction was not found
            if let Some(amount) = get_amount_from_tx_id(transactions, *tx) {
                resolve(account, amount);
            }
        }
        Transaction {
            client: _,
            tx,
            tx_type: TransactionType::ChargeBack,
        } => {
            // We ignore cases where the transaction was not found
            if let Some(amount) = get_amount_from_tx_id(transactions, *tx) {
                chargeback(account, amount);
            }
        }
    }
}

/// Returns the an option amount for a given transaction. Only deposits and withdrawals have specified amounts. For
/// these cases we return Some(amount). For disputes, resolves, and chargebacks we return None.
fn get_amount_from_tx_id(transactions: &Vec<Transaction>, tx_id: u32) -> Option<f32> {
    if let Some(transaction) = transactions.iter().find(|t| t.tx == tx_id) {
        match transaction {
            Transaction {
                client: _,
                tx: _,
                tx_type: TransactionType::Deposit(amount),
            } => Some(*amount),
            Transaction {
                client: _,
                tx: _,
                tx_type: TransactionType::WithDrawal(amount),
            } => Some(*amount),
            _ => None,
        }
    } else {
        None
    }
}

/// A deposit increases available and total funds with specified amount
fn deposit(account: &mut Account, amount: f32) {
    account.available += amount;
    account.total += amount;
}

/// A withdrawal decreases available and total funds with specified amount
/// If the withdrawal amount is greater than the available funds nothing happens
fn withdraw(account: &mut Account, amount: f32) {
    if amount < account.available {
        account.available -= amount;
        account.total -= amount;
    }
}

/// A dispute takes specified amount from available funds and puts it in held funds
/// Total funds remain the same.
fn dispute(account: &mut Account, amount: f32) {
    account.available -= amount;
    account.held += amount;
}

/// A resolve puts back the held funds into available funds
/// Total funds remain the same.
fn resolve(account: &mut Account, amount: f32) {
    account.available += amount;
    account.held -= amount;
}

/// A chargeback takes the disputed amount from held and total funds
/// and lockes the account
fn chargeback(account: &mut Account, amount: f32) {
    account.held -= amount;
    account.total -= amount;
    account.locked = true;
}

/// This function tries to read  and parse any existing accounts.csv from file
/// If nothing is found we create an empty accounts record
fn get_existing_accounts_data_from_file() -> Vec<Account> {
    let accounts: Vec<Account>;
    match std::fs::read_to_string("accounts.csv") {
        Ok(csv) => accounts = parse_accounts_csv(csv),
        Err(_) => {
            return vec![];
        }
    }

    accounts
}

/// Creates a Vec<Account> from csv and then drops the csv string
fn parse_accounts_csv(mut csv: String) -> Vec<Account> {
    csv.retain(|c| c != ' '); // Take out whitespaces
    let mut accounts: Vec<Account> = vec![];
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    for row in reader.deserialize() {
        // Ignore rows which cannot be parsed into an account
        if let Ok(account) = row{
            accounts.push(account)
        }
    }
    accounts
}

/// Represents a transaction. 
struct Transaction {
    client: u16,
    tx: u32,
    tx_type: TransactionType,
}

impl Transaction {
    /// This function creates a Transaction instance from a string line representing one row
    /// in the transaction.csv and drops it.
    fn new(mut s: String) -> Option<Transaction> {
        s.retain(|c| c != ' '); //Remove whitespaces
        let row: Vec<&str> = s.split(",").collect();
        let transaction_type = row[0];
        let client_id = row[1].parse::<u16>();
        let tx_id = row[2].parse::<u32>();
        let tx_amount = row[3].parse::<f32>();

        // We have nested matching in order to catch errors in parsing the strings to numbers
        // If parsing fails we return None, in which case we do not process the transaction further.
        match client_id {
            Ok(client) => match tx_id {
                Ok(tx) => match transaction_type {
                    "deposit" => match tx_amount {
                        Ok(amount) => {
                            let tx_type = TransactionType::Deposit(amount);
                            return Some(Transaction {
                                client,
                                tx,
                                tx_type,
                            });
                        }
                        Err(_) => {
                            return None;
                        }
                    },
                    "withdrawal" => match tx_amount {
                        Ok(amount) => {
                            let tx_type = TransactionType::WithDrawal(amount);
                            return Some(Transaction {
                                client,
                                tx,
                                tx_type,
                            });
                        }
                        Err(_) => {
                            return None;
                        }
                    },
                    "dispute" => {
                        let tx_type = TransactionType::Dispute;
                        return Some(Transaction {
                            client,
                            tx,
                            tx_type,
                        });
                    }
                    "resolve" => {
                        let tx_type = TransactionType::Resolve;
                        return Some(Transaction {
                            client,
                            tx,
                            tx_type,
                        });
                    }
                    "chargeback" => {
                        let tx_type = TransactionType::ChargeBack;
                        return Some(Transaction {
                            client,
                            tx,
                            tx_type,
                        });
                    }
                    _ => {
                        return None;
                    }
                },
                Err(_) => {
                    return None;
                }
            },
            Err(_) => {
                return None;
            }
        }
    }
}

/// Enum representing the different transaction variants
/// Deposits and WithDrawal variants hold the amount while
/// in the other cases we need to fetch the amount from
/// the transaction id, tx.
enum TransactionType {
    Deposit(f32),
    WithDrawal(f32),
    Dispute,
    Resolve,
    ChargeBack,
}

/// Holds all reveant account information of client.
#[derive(Serialize, Deserialize, Debug)]
struct Account {
    client: u16,
    available: f32,
    held: f32,
    total: f32, // This is not needed I think!
    locked: bool,
}

impl Account {
    /// We assume that a new Account is created with zero funds, zero held, and not locked
    fn new(client: u16) -> Account {
        Account {
            client,
            available: 0.,
            held: 0.,
            total: 0.,
            locked: false,
        }
    }
}
