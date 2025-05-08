# Recall: DevOps
const op_vault = "kesyjlvxpagq52ss4fz4evgzay"

export def read-secret [path] {
  op read $"op://($op_vault)/($path)"
}

# Creates a 1pass item in "RecallDevOps" vault and returns its ID
export def create-item [title] {
  op item create --vault $op_vault --category "Secure Note" --title $title --format json | from json | get id
}

# Write wallet address and private key into 1pass -> RecallDevOps -> testnet-wallets
export def write-wallet-credentials [item_id: string, name: string, wallet] {
  op item edit --vault $op_vault $item_id $"wallets.($name)-address[Text]=($wallet.address)" $"wallets.($name)-private-key=($wallet.private_key)"
}

export def write-contract-address [item_id: string, contract_name: string, address: string] {
  op item edit --vault $op_vault $item_id $"contracts.($contract_name)[Text]=($address)"
}

