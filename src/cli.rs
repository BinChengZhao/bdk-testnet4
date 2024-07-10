use clap::Subcommand;
#[derive(clap::Parser, Debug, Clone)]
#[clap(
    version = "1.0",
    author = "BinCheng",
    after_help = r#"Examples

## Create address
$ ./target/debug/bdk-testnet4 create-address --esplora-address 

## Get balance
$ ./target/debug/bdk-testnet4 get-balance

## List transactions
$ ./target/debug/bdk-testnet4 list-transactions

## Send transaction (The private key descriptor is hidden in the example. Please provide it when you use it)
$ ./target/debug/bdk-testnet4 pay -r bcrt1qwwf3ckm89aqxzpxhp62ee65s75kn7fnuk0y82g -a 10000

## Create descriptor
$ ./target/debug/bdk-testnet4 create-descriptor

"#
)]
#[clap(version = "1.0", author = "AtlasGraph Authors")]
pub struct Wallet {
    #[clap(
        short,
        long,
        default_value = "http://127.0.0.1:3000",
        value_name = "HOST:PORT",
        help = "Bitcoin esplora-testnet4 server address"
    )]
    pub esplora_address: String,
    #[clap(short, long, help = "Wallet descriptor", default_value = "")]
    pub descriptor: String,
    #[clap(short, long, help = "Change descriptor", default_value = "")]
    pub change_descriptor: String,
    #[command(subcommand)]
    pub commands: Commands,
}

// Bitcoin wallet supported subcommands
// 1. New descriptor, create new descriptor
// 2. Create address, generate new address for receiving
// 3. Get balance, get wallet balance
// 4. List transaction, list all transactions
// 5. Send transaction, send bitcoin to receiver address
// defalut subcommand is InitWallet
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    CreateDescriptor,
    CreateAddress,
    GetBalance,
    ListTransactions,
    Pay {
        #[clap(short, long, help = "Receiver address")]
        receiver: String,
        #[clap(short, long, help = "Amount to send")]
        amount: u64,
    },
}
