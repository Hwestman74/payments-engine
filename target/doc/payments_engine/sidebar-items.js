initSidebarItems({"enum":[["TransactionType","Enum representing the different transaction variants Deposits and WithDrawal variants hold the amount while in the other cases we need to fetch the amount from  the transaction id, tx."]],"fn":[["chargeback","A chargeback takes the disputed amount from held and total funds and lockes the account"],["deposit","A deposit increases available and total funds with specified amount"],["dispute","A dispute takes specified amount from available funds and puts it in held funds Total funds remain the same."],["get_amount_from_tx_id","Returns the an option amount for a given transaction. Only deposits and withdrawals have specified amounts. For these cases we return Some(amount). For disputes, resolves, and chargebacks we return None."],["get_existing_accounts_data_from_file","This function tries to read  and parse any existing accounts.csv from file If nothing is found we create an empty accounts record"],["main",""],["parse_accounts_csv",""],["process_transaction","This function is responsible for processing all the transactions and making updates to the accounts"],["resolve","A resolve puts back the held funds into available funds Total funds remain the same."],["round_to_decimals",""],["withdraw","A withdrawal decreases available and total funds with specified amount If the withdrawal amount is greater than the available funds nothing happens"]],"struct":[["Account","Hwe3olds all reveant account information of client."],["Transaction",""]]});