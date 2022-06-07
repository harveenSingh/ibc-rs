use ibc::applications::ics29_fee::msgs::register_counterparty::build_register_counterparty_address_message;
use ibc::core::ics24_host::identifier::ChannelId;
use ibc::signer::Signer;

use crate::chain::cosmos::query::account::get_or_fetch_account;
use crate::chain::cosmos::query::fee::query_counterparty_address;
use crate::chain::cosmos::retry::send_tx_with_account_sequence_retry;
use crate::chain::cosmos::types::account::Account;
use crate::chain::cosmos::types::config::TxConfig;
use crate::chain::cosmos::wait::wait_tx_succeed;
use crate::config::types::Memo;
use crate::error::Error;
use crate::keyring::KeyEntry;

pub async fn maybe_register_counterparty_address(
    tx_config: &TxConfig,
    key_entry: &KeyEntry,
    m_account: &mut Option<Account>,
    tx_memo: &Memo,
    channel_id: &ChannelId,
    address: &Signer,
    counterparty_address: &Signer,
) -> Result<(), Error> {
    let account =
        get_or_fetch_account(&tx_config.grpc_address, &key_entry.account, m_account).await?;

    let current_counterparty_address =
        query_counterparty_address(&tx_config.grpc_address, channel_id, address).await?;

    match &current_counterparty_address {
        Some(current_counterparty_address)
            if current_counterparty_address == counterparty_address.as_ref() =>
        {
            Ok(())
        }
        _ => {
            let message = build_register_counterparty_address_message(
                address,
                counterparty_address,
                channel_id,
            )
            .map_err(Error::ics29)?;

            let response = send_tx_with_account_sequence_retry(
                tx_config,
                key_entry,
                account,
                tx_memo,
                vec![message],
            )
            .await?;

            wait_tx_succeed(
                &tx_config.rpc_client,
                &tx_config.rpc_address,
                &tx_config.rpc_timeout,
                &response.hash,
            )
            .await?;

            Ok(())
        }
    }
}