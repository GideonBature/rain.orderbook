// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod error;
pub mod shared_state;
pub mod toast;
pub mod transaction_status;
pub mod types;

mod commands;
use commands::app::get_app_commit_sha;
use commands::authoring_meta::get_authoring_meta_v2_for_scenarios;
use commands::chain::{get_block_number, get_chainid};
use commands::charts::{make_charts, make_deployment_debug};
use commands::config::{
    check_dotrain_with_settings_errors, check_settings_errors, get_deployments, get_scenarios,
};
use commands::dotrain::parse_dotrain;
use commands::dotrain_add_order_lsp::{call_lsp_completion, call_lsp_hover, call_lsp_problems};
use commands::order::{
    compose_from_scenario, order_add, order_add_calldata, order_remove, order_remove_calldata,
    orders_list_write_csv, validate_spec_version,
};
use commands::order_quote::debug_order_quote;
use commands::order_take::order_trades_list_write_csv;
use commands::trade_debug::debug_trade;
use commands::vault::{
    vault_balance_changes_list_write_csv, vault_deposit, vault_deposit_approve_calldata,
    vault_deposit_calldata, vault_withdraw, vault_withdraw_calldata, vaults_list_write_csv,
};
use commands::wallet::get_address_from_ledger;
use shared_state::SharedState;

fn main() {
    if std::env::consts::OS == "linux" {
        // Disable webkitgtk Accelerated Compositing to avoid a blank screen
        // See https://github.com/tauri-apps/tauri/issues/5143
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
        run_tauri_app();
        std::env::remove_var("WEBKIT_DISABLE_COMPOSITING_MODE");
    } else {
        run_tauri_app();
    }
}

fn run_tauri_app() {
    tauri::Builder::default()
        .manage(SharedState::default())
        .invoke_handler(tauri::generate_handler![
            vaults_list_write_csv,
            vault_balance_changes_list_write_csv,
            vault_deposit,
            vault_withdraw,
            orders_list_write_csv,
            order_add,
            order_remove,
            order_trades_list_write_csv,
            get_address_from_ledger,
            get_chainid,
            get_block_number,
            parse_dotrain,
            call_lsp_completion,
            call_lsp_hover,
            call_lsp_problems,
            make_charts,
            make_deployment_debug,
            order_add_calldata,
            order_remove_calldata,
            vault_deposit_approve_calldata,
            vault_deposit_calldata,
            vault_withdraw_calldata,
            get_authoring_meta_v2_for_scenarios,
            compose_from_scenario,
            debug_order_quote,
            debug_trade,
            get_app_commit_sha,
            validate_spec_version,
            get_deployments,
            get_scenarios,
            check_settings_errors,
            check_dotrain_with_settings_errors
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
