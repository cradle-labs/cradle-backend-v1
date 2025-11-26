use anyhow::Result;
use colored::Colorize;
use std::io::Write;

use cradle_back_end::action_router::{ActionRouterInput, ActionRouterOutput};
use cradle_back_end::asset_book::db_types::AssetType;
use cradle_back_end::asset_book::processor_enums::{
    AssetBookProcessorInput, CreateExistingAssetInputArgs, CreateNewAssetInputArgs,
    GetAssetInputArgs,
};
use cradle_back_end::cli_helper::{call_action_router, execute_with_retry, initialize_app_config};
use cradle_back_end::cli_utils::{
    formatting::{format_decimal, format_record, format_table, print_header, print_section},
    input::Input,
    menu::Operation,
    print_info, print_success,
};

#[tokio::main]
async fn main() -> Result<()> {
    eprintln!(
        "{}",
        "╔═══════════════════════════════════════════════════════╗".bright_cyan()
    );
    eprintln!(
        "{}",
        "║         Cradle Asset Book Management CLI              ║".bright_cyan()
    );
    eprintln!(
        "{}",
        "╚═══════════════════════════════════════════════════════╝".bright_cyan()
    );
    eprintln!();

    // Initialize app config
    eprint!("Initializing app config... ");
    std::io::stderr().flush().ok();

    let app_config = match initialize_app_config() {
        Ok(config) => {
            eprintln!("{}", "✓ Ready".green());
            config
        }
        Err(e) => {
            eprintln!("{}", "✗ Failed".red());
            eprintln!("Error: {}", e);
            return Err(e);
        }
    };

    eprintln!();

    // Main loop
    loop {
        match Operation::select() {
            Ok(op) => match op {
                Operation::List => list_assets(&app_config).await?,
                Operation::View => view_asset(&app_config).await?,
                Operation::Create => create_asset(&app_config).await?,
                Operation::Update => update_asset(&app_config).await?,
                Operation::Delete => delete_asset(&app_config).await?,
                Operation::Cancel => {
                    eprintln!("{}", "Goodbye!".bright_cyan());
                    break;
                }
                _ => unimplemented!(),
            },
            Err(e) => {
                eprintln!("{}", format!("Error: {}", e).red());
                break;
            }
        }

        eprintln!();
    }

    Ok(())
}

async fn list_assets(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("List Assets");

    // Get optional type filter
    let type_opts = vec![
        "All",
        "Bridged",
        "Native",
        "Yield Bearing",
        "Chain Native",
        "StableCoin",
        "Volatile",
    ];
    let selected_type = Input::select_from_list("Filter by type", type_opts)?;

    // TODO: Implement filtered list when bulk query available
    print_info("Asset listing template (bulk query not yet available in processor)");

    Ok(())
}

async fn view_asset(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("View Asset");

    println!("Query by:");
    let query_opts = vec!["ID", "Token", "Asset Manager"];
    let query_type = Input::select_from_list("", query_opts)?;

    // Collect user input before async block
    let (asset_id_opt, asset_token_opt, asset_manager_opt) = match query_type {
        0 => {
            let id = Input::get_uuid("Enter asset ID")?;
            (Some(id), None, None)
        }
        1 => {
            let token = Input::get_string("Enter token address")?;
            (None, Some(token), None)
        }
        2 => {
            let manager = Input::get_string("Enter asset manager")?;
            (None, None, Some(manager))
        }
        _ => {
            let id = Input::get_uuid("Enter asset ID")?;
            (Some(id), None, None)
        }
    };

    execute_with_retry(
        || async {
            let asset_query = if let Some(id) = asset_id_opt {
                GetAssetInputArgs::ById(id)
            } else if let Some(token) = asset_token_opt.clone() {
                GetAssetInputArgs::ByToken(token)
            } else if let Some(manager) = asset_manager_opt.clone() {
                GetAssetInputArgs::ByAssetManager(manager)
            } else {
                return Err(anyhow::anyhow!("No query parameters provided"));
            };

            let input = AssetBookProcessorInput::GetAsset(asset_query);
            let router_input = ActionRouterInput::AssetBook(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::AssetBook(output) => {
                    print_success("Asset retrieved successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "view_asset",
    )
    .await?;

    Ok(())
}

async fn create_asset(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Create Asset");

    let creation_opts = vec!["New Asset", "Existing Asset"];
    let creation_type = Input::select_from_list("Asset type", creation_opts)?;

    match creation_type {
        0 => create_new_asset(app_config).await?,
        1 => create_existing_asset(app_config).await?,
        _ => create_new_asset(app_config).await?,
    }

    Ok(())
}

async fn create_new_asset(
    app_config: &cradle_back_end::utils::app_config::AppConfig,
) -> Result<()> {
    print_header("Create New Asset");

    let asset_name = Input::get_string("Asset name")?;
    let symbol = Input::get_string("Asset symbol")?;
    let decimals = Input::get_i64("Decimals (typically 8-18)")? as i32;

    // Select asset type
    let type_options = vec![
        "Bridged",
        "Native",
        "Yield Bearing",
        "Chain Native",
        "StableCoin",
        "Volatile",
    ];
    let selected_type = Input::select_from_list("Asset type", type_options)?;

    let asset_type = match selected_type {
        0 => AssetType::Bridged,
        1 => AssetType::Native,
        2 => AssetType::YieldBearing,
        3 => AssetType::ChainNative,
        4 => AssetType::StableCoin,
        5 => AssetType::Volatile,
        _ => AssetType::Native,
    };

    let icon = Input::get_optional_string("Icon URL")?.unwrap_or_default();

    execute_with_retry(
        || async {
            let create_input = CreateNewAssetInputArgs {
                asset_type: asset_type.clone(),
                name: asset_name.clone(),
                symbol: symbol.clone(),
                decimals,
                icon: icon.clone(),
            };

            let input = AssetBookProcessorInput::CreateNewAsset(create_input);
            let router_input = ActionRouterInput::AssetBook(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::AssetBook(output) => {
                    print_success("Asset created successfully via contract");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "create_new_asset",
    )
    .await?;

    Ok(())
}

async fn create_existing_asset(
    app_config: &cradle_back_end::utils::app_config::AppConfig,
) -> Result<()> {
    print_header("Create Existing Asset");

    let token = Input::get_string("Token address")?;
    let manager = Input::get_string("Asset manager")?;
    let asset_name = Input::get_string("Asset name")?;
    let symbol = Input::get_string("Asset symbol")?;
    let decimals = Input::get_i64("Decimals")? as i32;

    // Select asset type
    let type_options = vec![
        "Bridged",
        "Native",
        "Yield Bearing",
        "Chain Native",
        "StableCoin",
        "Volatile",
    ];
    let selected_type = Input::select_from_list("Asset type", type_options)?;

    let asset_type = match selected_type {
        0 => AssetType::Bridged,
        1 => AssetType::Native,
        2 => AssetType::YieldBearing,
        3 => AssetType::ChainNative,
        4 => AssetType::StableCoin,
        5 => AssetType::Volatile,
        _ => AssetType::Native,
    };

    let icon = Input::get_optional_string("Icon URL")?.unwrap_or_default();

    execute_with_retry(
        || async {
            let create_input = CreateExistingAssetInputArgs {
                asset_manager: Some(manager.clone()),
                token: token.clone(),
                asset_type: asset_type.clone(),
                name: asset_name.clone(),
                symbol: symbol.clone(),
                decimals,
                icon: icon.clone(),
            };

            let input = AssetBookProcessorInput::CreateExistingAsset(create_input);
            let router_input = ActionRouterInput::AssetBook(input);

            match call_action_router(router_input, app_config.clone()).await? {
                ActionRouterOutput::AssetBook(output) => {
                    print_success("Asset registered successfully");
                    Ok(())
                }
                _ => Err(anyhow::anyhow!("Unexpected output type")),
            }
        },
        "create_existing_asset",
    )
    .await?;

    Ok(())
}

async fn update_asset(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Update Asset");

    print_info("Asset book processor does not support update operations");

    Ok(())
}

async fn delete_asset(app_config: &cradle_back_end::utils::app_config::AppConfig) -> Result<()> {
    print_header("Delete Asset");

    print_info("Asset book processor does not support delete operations");

    Ok(())
}
