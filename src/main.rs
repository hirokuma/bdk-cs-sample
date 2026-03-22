use bdk_electrum::electrum_client::Client;
use bdk_electrum::{BdkElectrumClient, electrum_client};
use bdk_wallet::KeychainKind;
use bdk_wallet::PersistedWallet;
use bdk_wallet::Wallet;
use bdk_wallet::bitcoin::{Network, bip32};
use bdk_wallet::chain::spk_client::{SyncRequest, SyncRequestBuilder};
use bdk_wallet::keys::{GeneratableKey, GeneratedKey};
use bdk_wallet::rusqlite::Connection;
use bdk_wallet::{AddressInfo, miniscript};
use std::path::Path;

const STOP_GAP: usize = 50;
const BATCH_SIZE: usize = 5;
const NETWORK: Network = Network::Regtest;
const ELECTRUM_HOST: &str = "tcp://localhost:50001";
const XPRV_FNAME: &str = "./wallet-data/wallet.xprv";
const WALLET_FNAME: &str = "./wallet-data/wallet.bdk";

fn main() -> anyhow::Result<()> {
    if !Path::new(XPRV_FNAME).exists() {
        create_xprv(XPRV_FNAME);
    }
    let mut wallet = create_or_load_wallet(WALLET_FNAME, XPRV_FNAME);

    let address: AddressInfo = wallet.reveal_next_address(KeychainKind::External);
    println!(
        "Generated address {} at index {}",
        address.address, address.index
    );

    // Create the Electrum client
    let client: BdkElectrumClient<Client> =
        BdkElectrumClient::new(electrum_client::Client::new(ELECTRUM_HOST).unwrap());

    // Perform the initial full scan on the wallet
    full_scan(&mut wallet, &client);

    let mut prev_balance = wallet.balance();
    println!("Wallet balance: {} sat", prev_balance.total().to_sat());

    loop {
        std::thread::sleep(std::time::Duration::from_secs(10));

        sync(&mut wallet, &client);
        let balance = wallet.balance();
        if prev_balance != balance {
            println!(
                "Wallet balance changed: {} --> {} sat",
                prev_balance.total().to_sat(),
                balance.total().to_sat()
            );
            prev_balance = balance;
        }
    }
}

fn create_xprv(fname: &str) {
    let xprv_base: GeneratedKey<bip32::Xpriv, miniscript::Tap> =
        bip32::Xpriv::generate(()).unwrap();
    let mut xprv_base = xprv_base.into_key();
    xprv_base.network = NETWORK.into();

    // XXX 秘密鍵をプレーンテキスト形式で保存しているので注意
    std::fs::write(fname, xprv_base.to_string()).unwrap();
}

fn create_or_load_wallet(wallet_fname: &str, xprv_fname: &str) -> PersistedWallet<Connection> {
    let xprv = std::fs::read_to_string(xprv_fname).unwrap();

    let mut conn = Connection::open(wallet_fname).unwrap();
    let xprv_extn = format!("tr({}/86'/1'/0/0/*)", xprv);
    let xprv_intr = format!("tr({}/86'/1'/0/1/*)", xprv);
    let wallet_opt = Wallet::load()
        .descriptor(KeychainKind::External, Some(xprv_extn.clone()))
        .descriptor(KeychainKind::Internal, Some(xprv_intr.clone()))
        .extract_keys()
        .check_network(NETWORK)
        .load_wallet(&mut conn)
        .unwrap();

    let wallet: PersistedWallet<Connection> = match wallet_opt {
        Some(wallet) => {
            println!("Loaded existing wallet database.");
            wallet
        }
        None => {
            println!("Creating new wallet database.");
            Wallet::create(xprv_extn, xprv_intr)
                .network(NETWORK)
                .create_wallet(&mut conn)
                .unwrap()
        }
    };
    wallet
}

fn full_scan(wallet: &mut PersistedWallet<Connection>, client: &BdkElectrumClient<Client>) {
    let full_scan_request = wallet.start_full_scan();
    let update = client
        .full_scan(full_scan_request, STOP_GAP, BATCH_SIZE, true)
        .unwrap();
    wallet.apply_update(update).unwrap();
}

fn sync(wallet: &mut PersistedWallet<Connection>, client: &BdkElectrumClient<Client>) {
    let sync_request = sync_request(wallet);
    let sync_response = client.sync(sync_request, BATCH_SIZE, false).unwrap();
    wallet.apply_update(sync_response).unwrap();
}

fn sync_request(wallet: &Wallet) -> SyncRequestBuilder<(bdk_wallet::KeychainKind, u32)> {
    let mut spks_to_sync = std::collections::BTreeSet::new();

    // Externalアドレスのみチェックする
    if let Some(derived_index) = wallet.derivation_index(KeychainKind::External) {
        for index in 0..=derived_index {
            let address_info = wallet.peek_address(KeychainKind::External, index);
            spks_to_sync.insert((
                (KeychainKind::External, index),
                address_info.address.script_pubkey(),
            ));
        }
    }
    for tx in wallet.transactions() {
        if tx.chain_position.is_unconfirmed() {
            for out in &tx.tx_node.tx.output {
                if let Some(index) = wallet.spk_index().index_of_spk(out.script_pubkey.clone()) {
                    spks_to_sync.insert((*index, out.script_pubkey.clone()));
                }
            }
        }
    }
    let chain_tip = wallet.local_chain().tip();
    SyncRequest::builder()
        .chain_tip(chain_tip)
        .spks_with_indexes(spks_to_sync)
}
