use std::{collections::BTreeSet, io::Write, str::FromStr};

use anyhow::Result;
use bdk_esplora::EsploraExt;
use bdk_wallet::{miniscript, wallet::Wallet, KeychainKind, SignOptions};
use bitcoin::{Address, Amount, BlockHash, Network};
use clap::Parser;

mod cli;
use cli::{Commands, Wallet as Testnet4Wallet};

/// Testnet4 genesis hash
pub const TESTNET4_GENESIS_HASH: &str =
    "00000000da84f2bafbbc53dee25a72ae507ff4914b867c565be350b0da8bf043";

fn main() -> Result<()> {
    let wallet = Testnet4Wallet::parse();
    wallet.dispatch_command()?;
    Ok(())
}

/// Implementation of the `Testnet4Wallet` struct.
impl Testnet4Wallet {
    /// Dispatches the command based on the value of `self.commands`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the command is successfully dispatched, otherwise returns an error.
    pub fn dispatch_command(&self) -> Result<()> {
        match &self.commands {
            Commands::CreateDescriptor => {
                println!("New Descriptor \r\n");
                self.new_descriptor()?;
            }
            Commands::CreateAddress => {
                println!("Create address \r\n");
                self.create_address()?;
            }
            Commands::GetBalance => {
                println!("Get balance \r\n");
                self.get_balance()?;
            }
            Commands::ListTransactions => {
                println!("List transaction \r\n");
                self.list_transactions()?;
            }
            Commands::Pay { receiver, amount } => {
                println!("Send transaction \r\n");
                self.pay(receiver, *amount)?;
            }
        };
        Ok(())
    }

    /// Generates a new descriptor and prints the private and public descriptors.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the descriptors are successfully generated and printed, otherwise returns an error.
    fn new_descriptor(&self) -> Result<()> {
        // first generate a new private key
        let int_private_key = bitcoin::key::PrivateKey::generate(Network::Testnet);
        let ext_private_key = bitcoin::key::PrivateKey::generate(Network::Testnet);

        let int_priv_descriptor = format!("wpkh({})", int_private_key.to_string());
        let ext_priv_descriptor = format!("wpkh({})", ext_private_key.to_string());

        let (int_pub_descriptor, ..) = bdk_wallet::descriptor!(wpkh(int_private_key))?;
        let (ext_pub_descriptor, ..) = bdk_wallet::descriptor!(wpkh(ext_private_key))?;

        println!(
            "Generated priv-descriptor: internal: {} \r\n",
            int_priv_descriptor
        );
        println!(
            "Generated priv-descriptor: external: {} \r\n",
            ext_priv_descriptor
        );

        println!(
            "Generated pub-descriptor: internal: {} \r\n",
            int_pub_descriptor
        );
        println!(
            "Generated pub-descriptor: external: {} \r\n",
            ext_pub_descriptor
        );

        Ok(())
    }

    /// Creates a new address for receiving bitcoin.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the address is successfully generated and printed, otherwise returns an error.
    fn create_address(&self) -> Result<()> {
        let mut wallet = self.wallet()?;
        let address = wallet.next_unused_address(KeychainKind::External);
        println!("Generated Address: {}", address);
        Ok(())
    }

    /// Gets the wallet balance and prints it.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the balance is successfully retrieved and printed, otherwise returns an error.
    fn get_balance(&self) -> Result<()> {
        let wallet = self.wallet()?;
        let balance = wallet.balance();
        println!(
            " \r\n Wallet balance after syncing: \r\n {:?} \r\n",
            balance
        );
        Ok(())
    }

    /// Lists all transactions and prints their transaction IDs.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the transactions are successfully listed and printed, otherwise returns an error.
    fn list_transactions(&self) -> Result<()> {
        let wallet = self.wallet()?;
        for tx in wallet.tx_graph().full_txs() {
            println!("{:?} \r\n", tx.txid);
        }
        Ok(())
    }

    /// Sends bitcoin to the receiver address.
    ///
    /// # Arguments
    ///
    /// * `receiver` - The receiver's address.
    /// * `amount` - The amount of bitcoin to send.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the transaction is successfully sent, otherwise returns an error.
    fn pay(&self, receiver: &String, amount: u64) -> Result<()> {
        let mut wallet = self.wallet()?;
        let balance = wallet.balance();
        let amount = Amount::from_sat(amount);

        if balance.total() < amount {
            println!("balance: {}, amount: {}", balance.total(), amount);
            anyhow::bail!("Insufficient balance");
        }

        let faucet_address = Address::from_str(receiver)?.require_network(Network::Testnet)?;

        let mut tx_builder = wallet.build_tx();
        tx_builder
            .add_recipient(faucet_address.script_pubkey(), amount)
            .enable_rbf();

        // TODO: what is psbt?
        let mut psbt = tx_builder.finish()?;

        let finalized = wallet.sign(&mut psbt, SignOptions::default())?;
        if !finalized {
            anyhow::bail!("Transaction not finalized");
        }

        let tx = psbt.extract_tx()?;

        // broadcast the transaction
        self.client()?.broadcast(&tx)?;
        println!("\r\n Transaction sent: {:?} \r\n", tx.compute_txid());
        Ok(())
    }

    /// Returns a blocking Esplora client.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the blocking Esplora client if successful, otherwise returns an error.
    fn client(&self) -> Result<bdk_esplora::esplora_client::BlockingClient> {
        let esplora_address = self.esplora_address.as_str();
        let client = bdk_esplora::esplora_client::Builder::new(esplora_address).build_blocking();
        Ok(client)
    }

    /// Returns a wallet instance.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the wallet instance if successful, otherwise returns an error.
    fn wallet(&self) -> Result<Wallet> {
        let (descriptor, change_descriptor) = self.descriptor()?;
        let testnet4_genesis_hash: BlockHash =
            BlockHash::from_str(TESTNET4_GENESIS_HASH).expect("must be valid hash");
        let mut wallet = Wallet::new_with_genesis_hash(
            &descriptor,
            &change_descriptor,
            Network::Testnet,
            testnet4_genesis_hash,
        )?;

        print!("Syncing... \r\n");
        let client =
            bdk_esplora::esplora_client::Builder::new(&self.esplora_address).build_blocking();

        let request = wallet.start_full_scan().inspect_spks_for_all_keychains({
            let mut once = BTreeSet::<KeychainKind>::new();
            move |keychain, spk_i, s| {
                match once.insert(keychain) {
                    true => print!(
                        "\n Scanning keychain [{:?}], spk_i: {}, script: {}",
                        keychain, spk_i, s
                    ),
                    false => print!(" {:<3} \r\n", spk_i),
                };
                std::io::stdout().flush().expect("must flush")
            }
        });

        let mut update = client.full_scan(request, 20, 2)?;
        let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
        let _ = update.graph_update.update_last_seen_unconfirmed(now);

        wallet.apply_update(update)?;

        // For teminal clean
        println!("\r\n");

        Ok(wallet)
    }

    /// Returns the descriptor strings.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the descriptor strings if successful, otherwise returns an error.
    fn descriptor(&self) -> Result<(String, String)> {
        let mut desicrptor: &str = self.descriptor.as_str();
        let mut change_descriptor: &str = self.change_descriptor.as_str();

        if desicrptor.is_empty() {
            desicrptor =
                "wpkh(033d4dfd8a751eaaff139a03310612c78a8a0f2657122f875ba3f0ccde35c94b4a)#8m9vvd32";
        }
        if change_descriptor.is_empty() {
            change_descriptor =
                "wpkh(039f643230874ab9fdc79443578eee96ff682b1feb168d277269b152c2c4dcd562)#nnsy5ygv";
        }

        Ok((desicrptor.to_string(), change_descriptor.to_string()))
    }
}
