use cradle_back_end::accounts::db_types::{CradleAccountRecord, CradleWalletAccountRecord};
use cradle_back_end::market::db_types::{MarketRecord, MarketType, MarketStatus};
use cradle_back_end::order_book::db_types::{OrderBookRecord, OrderType};
use cradle_back_end::asset_book::db_types::AssetBookRecord;
use cradle_back_end::lending_pool::db_types::{LendingPoolRecord, LoanRecord};
use cradle_back_end::listing::db_types::{CradleNativeListingRow, CompanyRow, ListingStatus};
use bigdecimal::BigDecimal;
use uuid::Uuid;

pub fn base_layout(content: &str) -> String {
    format!(
         r##"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Cradle Admin Dashboard</title>
    <script src="https://unpkg.com/htmx.org@1.9.10"></script>
    <script src="https://cdn.tailwindcss.com"></script>
    <style>
        .sidebar-scroll::-webkit-scrollbar {{ width: 6px; }}
        .sidebar-scroll::-webkit-scrollbar-thumb {{ background-color: #4b5563; border-radius: 3px; }}
    </style>
</head>
<body class="bg-gray-900 text-gray-100 font-sans antialiased h-screen flex overflow-hidden">
    <div id="main-content" class="flex w-full h-full">
        {content}
    </div>
</body>
</html>
"##
    )
}

pub fn index_page() -> String {
    base_layout(
        r##"
        <div class="h-full w-64 bg-gray-800 border-r border-gray-700 flex flex-col" hx-get="/ui/accounts" hx-trigger="load" hx-swap="innerHTML">
            <!-- Sidebar content loads here -->
            <div class="p-4 text-center text-gray-400">Loading accounts...</div>
        </div>
        <div class="flex-1 flex flex-col items-center justify-center p-10 text-gray-500">
            <h1 class="text-3xl font-bold mb-4 text-white">Cradle Admin</h1>
            <p class="mb-8">Select an account from the sidebar to begin.</p>
            <div class="border-t border-gray-700 pt-8 mt-4">
                <p class="text-sm text-gray-400 mb-4">Or access global admin tools:</p>
                <a href="/ui/admin" class="bg-purple-600 hover:bg-purple-500 px-6 py-3 rounded-lg text-white font-bold transition-colors">
                    Admin Tools (Assets, Markets, Aggregator)
                </a>
            </div>
        </div>
        "##
    )
}

pub fn account_list(accounts: Vec<CradleWalletAccountRecord>) -> String {
    // Wrap in proper container to preserve sidebar structure
    let mut list_html = String::new();
    list_html.push_str(r##"<div class="p-4 border-b border-gray-700 font-bold text-lg bg-gray-800">Cradle Accounts</div><div class="flex-1 overflow-y-auto sidebar-scroll">"##);
    
    for acc in accounts {
        let short_id = if acc.address.len() > 10 {
            format!("{}...", &acc.address[0..10])
        } else {
             acc.address.clone()
        };
        
        list_html.push_str(&format!(
            r##"
            <div class="p-3 border-b border-gray-700 hover:bg-gray-700 cursor-pointer transition-colors"
                 hx-get="/ui/dashboard/{}"
                 hx-target="#main-content"
                 hx-push-url="true">
                <div class="font-medium text-blue-400">{}</div>
                <div class="text-xs text-gray-500 mt-1">{}</div>
            </div>
            "##,
            acc.id,
            short_id,
            acc.id
        ));
    }
    list_html.push_str("</div>");
    list_html
}

pub struct Balance {
    pub token: String,
    pub amount: String,
}

pub fn dashboard(account_id: Uuid, balances: Vec<Balance>) -> String {
     let mut balance_html = String::new();
     for b in balances {
         balance_html.push_str(&format!(
             r##"<div class="px-4 py-2 bg-gray-700/50 rounded-lg border border-gray-600">
                    <span class="text-gray-400 text-xs">{}</span>
                    <div class="font-bold text-green-400">{}</div>
                </div>"##,
             b.token, b.amount
         ));
     }

    format!(
        r##"
        <div class="h-full w-64 bg-gray-800 border-r border-gray-700 flex flex-col" hx-get="/ui/accounts" hx-trigger="load" hx-swap="innerHTML">
             <!-- Sidebar reloads -->
        </div>
        <div class="flex-1 flex flex-col min-w-0 bg-gray-900">
            <!-- Top Bar: Info & Balances -->
            <div class="bg-gray-800 border-b border-gray-700 px-6 py-4 flex items-center justify-between shadow-md z-10">
                <div>
                     <div class="text-xs text-gray-500 uppercase tracking-wider font-semibold">Active Account</div>
                     <div class="text-xl font-mono text-white">{}</div>
                </div>
                <div class="flex gap-3">
                    {balance_html}
                </div>
            </div>

            <!-- Tabs Navigation -->
            <div class="flex border-b border-gray-700 bg-gray-800/50 px-6 pt-2 gap-1">
                <button class="px-6 py-3 text-sm font-medium text-blue-400 border-b-2 border-blue-400 hover:bg-gray-700/50 rounded-t-lg transition-colors focus:outline-none"
                        hx-get="/ui/tabs/markets?account_id={}"
                        hx-target="#tab-content"
                        _="#tab-content"
                        class:selected="border-blue-400 text-blue-400"
                        class:not-selected="border-transparent text-gray-400 hover:text-gray-200">
                    Markets
                </button>
                <button class="px-6 py-3 text-sm font-medium text-gray-400 border-b-2 border-transparent hover:text-gray-200 hover:bg-gray-700/50 rounded-t-lg transition-colors focus:outline-none"
                        hx-get="/ui/tabs/onramp?account_id={}"
                        hx-target="#tab-content">
                    On-Ramp
                </button>
                <button class="px-6 py-3 text-sm font-medium text-gray-400 border-b-2 border-transparent hover:text-gray-200 hover:bg-gray-700/50 rounded-t-lg transition-colors focus:outline-none"
                        hx-get="/ui/tabs/faucet?account_id={}"
                        hx-target="#tab-content">
                    Faucet
                </button>
                <button class="px-6 py-3 text-sm font-medium text-gray-400 border-b-2 border-transparent hover:text-gray-200 hover:bg-gray-700/50 rounded-t-lg transition-colors focus:outline-none"
                        hx-get="/ui/tabs/lending?account_id={}"
                        hx-target="#tab-content">
                    Lending
                </button>
                <button class="px-6 py-3 text-sm font-medium text-gray-400 border-b-2 border-transparent hover:text-gray-200 hover:bg-gray-700/50 rounded-t-lg transition-colors focus:outline-none"
                        hx-get="/ui/tabs/listings?account_id={}"
                        hx-target="#tab-content">
                    Listings
                </button>
                <button class="px-6 py-3 text-sm font-medium text-gray-400 border-b-2 border-transparent hover:text-gray-200 hover:bg-gray-700/50 rounded-t-lg transition-colors focus:outline-none"
                        hx-get="/ui/tabs/oracle?account_id={}"
                        hx-target="#tab-content">
                    Oracle
                </button>
            </div>

            <!-- Tab Content Area -->
            <div id="tab-content" class="flex-1 overflow-y-auto p-6" hx-get="/ui/tabs/markets?account_id={}" hx-trigger="load">
                <div class="flex justify-center items-center h-full text-gray-500 animate-pulse">Loading Markets...</div>
            </div>
        </div>
        
        <script>
            // Simple tab switching logic for styles
            document.querySelectorAll('button[hx-get^="/ui/tabs"]').forEach(btn => {{
                btn.addEventListener('click', function() {{
                    // Reset all
                    this.parentElement.querySelectorAll('button').forEach(b => {{
                        b.classList.remove('border-blue-400', 'text-blue-400');
                        b.classList.add('border-transparent', 'text-gray-400');
                    }});
                    // Set active
                    this.classList.remove('border-transparent', 'text-gray-400');
                    this.classList.add('border-blue-400', 'text-blue-400');
                }});
            }});
        </script>
        "##,
        account_id,
        account_id, account_id, account_id, account_id, account_id, account_id, account_id
    )
}

pub fn markets_tab(account_id: Uuid, markets: Vec<MarketRecord>) -> String {
    // Add swap transition for snappy feel
    let mut market_opts = String::new();
    for m in markets {
        market_opts.push_str(&format!(
            r##"<option value="{}">{} ({:?})</option>"##,
            m.id, m.name, m.market_type
        ));
    }

    format!(
        r##"
        <div class="h-full flex flex-col gap-6">
            <!-- Market Selection -->
            <div class="bg-gray-800 p-4 rounded-xl border border-gray-700 shadow-sm">
                <div class="flex items-center gap-4">
                     <label class="text-sm font-medium text-gray-300">Select Market:</label>
                     <select class="bg-gray-900 border border-gray-600 text-gray-100 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 w-64"
                             hx-get="/ui/market_detail"
                             hx-target="#market-view"
                             hx-swap="innerHTML swap:200ms"
                             hx-include="[name='account_id']"
                             name="market_id">
                         <option value="">-- Choose a Market --</option>
                         {}
                     </select>
                     <input type="hidden" name="account_id" value="{}" />
                </div>
            </div>

            <!-- Market Details & Order Area -->
            <div id="market-view" class="flex-1 flex flex-col">
                <div class="flex items-center justify-center h-full text-gray-500 border-2 border-dashed border-gray-700 rounded-xl">
                    Select a market to view details and place orders
                </div>
            </div>
        </div>
        "##,
        market_opts,
        account_id
    )
}

pub fn market_detail(market: MarketRecord, account_id: Uuid, recent_orders: Vec<OrderBookRecord>) -> String {
    let mut orders_html = String::new();
    for o in recent_orders {
        // Determine side: Buy if asking for Asset One (Base)
        let is_buy = o.ask_asset == market.asset_two;
        let side_text = if is_buy { "Buy" } else { "Sell" };
        let side_color = if is_buy { "text-green-400" } else { "text-red-400" };

        orders_html.push_str(&format!(
            r##"
            <tr class="border-b border-gray-700 hover:bg-gray-700/50">
                <td class="px-4 py-3 font-mono text-sm {}">{}</td>
                <td class="px-4 py-3 text-sm">{}</td>
                <td class="px-4 py-3 text-sm">{}</td>
                <td class="px-4 py-3 text-sm">{}</td>
                <td class="px-4 py-3 text-sm">{}</td>
                <td class="px-4 py-3 text-xs text-gray-400">{}</td>
            </tr>
            "##,
            side_color,
            side_text,
            format!("{:?}", o.order_type),
             o.price,
             o.bid_amount,
             o.ask_amount,
             format!("{:?}", o.status)
        ));
    }

    // Fallback if empty
    if orders_html.is_empty() {
        orders_html = r#"<tr><td colspan="6" class="p-4 text-center text-gray-500 italic">No recent orders</td></tr>"#.to_string();
    }

    format!(
         r##"
         <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 h-full">
            <!-- Market Info & Place Order -->
            <div class="lg:col-span-1 space-y-6">
                 <!-- Info Card -->
                 <div class="bg-gray-800 p-6 rounded-xl border border-gray-700">
                     <h3 class="text-xl font-bold text-white mb-2">{name}</h3>
                     <p class="text-sm text-gray-400 mb-4">{description}</p>
                     <div class="grid grid-cols-2 gap-4 text-sm">
                        <div class="bg-gray-700/50 p-2 rounded">
                            <span class="block text-xs text-gray-500">Asset One (Base)</span>
                            <span class="font-mono text-gray-200" id="asset-one-id">{asset_one}</span>
                        </div>
                        <div class="bg-gray-700/50 p-2 rounded">
                             <span class="block text-xs text-gray-500">Asset Two (Quote)</span>
                            <span class="font-mono text-gray-200" id="asset-two-id">{asset_two}</span>
                        </div>
                     </div>
                 </div>

                 <!-- Order Form - Amount In/Out Style -->
                 <!-- SEMANTICS: ask_asset=what you GIVE, bid_asset=what you WANT -->
                 <div class="bg-gray-800 p-6 rounded-xl border border-gray-700">
                    <h4 class="text-lg font-bold text-gray-200 mb-4 border-b border-gray-600 pb-2">Place Order</h4>
                    <form id="order-form" hx-post="/ui/order" hx-target="#order-message">
                        <input type="hidden" name="account_id" value="{account_id}" />
                        <input type="hidden" name="market_id" value="{market_id}" />
                        <input type="hidden" id="asset_in" name="asset_in" value="{asset_one}" />
                        <input type="hidden" id="asset_out" name="asset_out" value="{asset_two}" />

                        <div class="space-y-4">
                            <div>
                                <label class="block text-xs font-medium text-gray-400 mb-1">Order Type</label>
                                <select id="order_type" name="order_type" class="w-full bg-gray-700 border-none rounded p-2 text-sm focus:ring-1 focus:ring-blue-500">
                                    <option value="limit">Limit Order</option>
                                    <option value="market">Market Order</option>
                                </select>
                            </div>

                            <!-- Amount In (What you give) -->
                            <div class="bg-gray-700/30 p-4 rounded-lg border border-gray-600">
                                <div class="flex justify-between items-center mb-2">
                                    <label class="text-xs font-medium text-gray-400">You Give (Amount In)</label>
                                    <select id="asset_in_select" class="bg-gray-600 border-none rounded px-2 py-1 text-xs">
                                        <option value="{asset_one}" data-symbol="Asset One">{asset_one_short}</option>
                                        <option value="{asset_two}" data-symbol="Asset Two">{asset_two_short}</option>
                                    </select>
                                </div>
                                <input type="text" id="amount_in" name="amount_in" placeholder="0.00"
                                       class="w-full bg-gray-900 border-none rounded p-3 text-lg font-mono focus:ring-1 focus:ring-blue-500" required>
                            </div>

                            <!-- Price (inferred or editable) -->
                            <div class="flex items-center justify-center gap-2 text-gray-400">
                                <div class="flex-1 h-px bg-gray-600"></div>
                                <div class="text-xs">
                                    Price: <span id="price-display" class="font-mono text-blue-400">--</span>
                                    <span id="price-unit" class="text-gray-500"></span>
                                </div>
                                <div class="flex-1 h-px bg-gray-600"></div>
                            </div>

                            <!-- Amount Out (What you receive) -->
                            <div class="bg-gray-700/30 p-4 rounded-lg border border-gray-600">
                                <div class="flex justify-between items-center mb-2">
                                    <label class="text-xs font-medium text-gray-400">You Receive (Amount Out)</label>
                                    <span id="asset_out_label" class="bg-gray-600 rounded px-2 py-1 text-xs">{asset_two_short}</span>
                                </div>
                                <input type="text" id="amount_out" name="amount_out" placeholder="0.00"
                                       class="w-full bg-gray-900 border-none rounded p-3 text-lg font-mono focus:ring-1 focus:ring-blue-500" required>
                            </div>

                            <!-- Price Input (for direct control) -->
                            <div id="price-input-section">
                                <label class="block text-xs font-medium text-gray-400 mb-1">Set Price (optional - adjusts Amount Out)</label>
                                <input type="text" id="price_input" name="price" placeholder="Auto-calculated"
                                       class="w-full bg-gray-700 border-none rounded p-2 text-sm font-mono focus:ring-1 focus:ring-blue-500">
                                <p class="text-xs text-gray-500 mt-1">Price = Amount In / Amount Out</p>
                            </div>

                            <button type="submit" class="w-full bg-blue-600 hover:bg-blue-500 text-white font-bold py-3 rounded-lg transition-colors">
                                Submit Order
                            </button>

                             <div id="order-message" class="text-center text-sm min-h-[20px]"></div>
                        </div>
                    </form>
                 </div>
            </div>

            <!-- Recent Orders (Right Side) -->
            <div class="lg:col-span-2 bg-gray-800 rounded-xl border border-gray-700 flex flex-col h-full overflow-hidden">
                <div class="p-4 border-b border-gray-700 flex justify-between items-center bg-gray-700/30">
                    <h4 class="font-bold text-gray-200">Recent Orders</h4>
                    <button class="text-xs text-blue-400 hover:text-blue-300"
                            hx-get="/ui/market_detail?market_id={market_id}&account_id={account_id}"
                            hx-target="#market-view">Refresh</button>
                </div>
                <div class="overflow-x-auto flex-1">
                    <table class="w-full text-left">
                        <thead class="bg-gray-700/50 text-xs text-gray-400 uppercase">
                            <tr>
                                <th class="px-4 py-2">Side</th>
                                <th class="px-4 py-2">Type</th>
                                <th class="px-4 py-2">Price</th>
                                <th class="px-4 py-2">Amount In</th>
                                <th class="px-4 py-2">Amount Out</th>
                                <th class="px-4 py-2">Status</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-700/50">
                            {orders_html}
                        </tbody>
                    </table>
                </div>
            </div>
         </div>

         <script>
         (function() {{
             const assetOne = '{asset_one}';
             const assetTwo = '{asset_two}';
             const assetOneShort = '{asset_one_short}';
             const assetTwoShort = '{asset_two_short}';

             const assetInSelect = document.getElementById('asset_in_select');
             const amountIn = document.getElementById('amount_in');
             const amountOut = document.getElementById('amount_out');
             const priceInput = document.getElementById('price_input');
             const priceDisplay = document.getElementById('price-display');
             const priceUnit = document.getElementById('price-unit');
             const assetOutLabel = document.getElementById('asset_out_label');
             const assetInHidden = document.getElementById('asset_in');
             const assetOutHidden = document.getElementById('asset_out');

             let lastEdited = 'amount_in'; // Track which field was last edited

             function updateAssets() {{
                 const selectedAssetIn = assetInSelect.value;
                 const selectedAssetOut = selectedAssetIn === assetOne ? assetTwo : assetOne;
                 const assetOutShort = selectedAssetIn === assetOne ? assetTwoShort : assetOneShort;

                 // Update hidden form fields:
                 // asset_in = what you GIVE (maps to ask_asset in order book)
                 // asset_out = what you WANT (maps to bid_asset in order book)
                 assetInHidden.value = selectedAssetIn;
                 assetOutHidden.value = selectedAssetOut;

                 // Update the "You Receive" label
                 assetOutLabel.textContent = assetOutShort;

                 // Update price unit display
                 const assetInShort = selectedAssetIn === assetOne ? assetOneShort : assetTwoShort;
                 priceUnit.textContent = `(${{assetInShort}}/${{assetOutShort}})`;

                 updatePrice();
             }}

             function updatePrice() {{
                 const inVal = parseFloat(amountIn.value) || 0;
                 const outVal = parseFloat(amountOut.value) || 0;

                 if (inVal > 0 && outVal > 0) {{
                     const price = inVal / outVal;
                     priceDisplay.textContent = price.toFixed(8);
                     if (!priceInput.dataset.userEdited) {{
                         priceInput.value = price.toFixed(8);
                     }}
                 }} else {{
                     priceDisplay.textContent = '--';
                 }}
             }}

             function calculateFromPrice() {{
                 const inVal = parseFloat(amountIn.value) || 0;
                 const priceVal = parseFloat(priceInput.value) || 0;

                 if (inVal > 0 && priceVal > 0) {{
                     const outVal = inVal / priceVal;
                     amountOut.value = outVal.toFixed(8);
                     priceDisplay.textContent = priceVal.toFixed(8);
                 }}
             }}

             assetInSelect.addEventListener('change', updateAssets);

             amountIn.addEventListener('input', function() {{
                 lastEdited = 'amount_in';
                 priceInput.dataset.userEdited = '';
                 updatePrice();
             }});

             amountOut.addEventListener('input', function() {{
                 lastEdited = 'amount_out';
                 priceInput.dataset.userEdited = '';
                 updatePrice();
             }});

             priceInput.addEventListener('input', function() {{
                 priceInput.dataset.userEdited = 'true';
                 calculateFromPrice();
             }});

             // Initialize
             updateAssets();
         }})();
         </script>
         "##,
         name = market.name,
         description = market.description.clone().unwrap_or_default(),
         asset_one = market.asset_one,
         asset_two = market.asset_two,
         asset_one_short = &market.asset_one.to_string()[..8],
         asset_two_short = &market.asset_two.to_string()[..8],
         account_id = account_id,
         market_id = market.id,
         orders_html = orders_html
    )
}

pub fn on_ramp_tab(account_id: Uuid) -> String {
    format!(
         r##"
        <div class="max-w-2xl mx-auto space-y-8">
            <div class="text-center">
                <h2 class="text-3xl font-bold text-white mb-2">Fiat On-Ramp</h2>
                <p class="text-gray-400">Purchase tokens directly using your preferred payment method.</p>
            </div>

            <div class="bg-gray-800 p-8 rounded-2xl border border-gray-700 shadow-xl">
                 <form hx-post="/ui/on_ramp" hx-target="#on-ramp-result" class="space-y-6">
                    <input type="hidden" name="account_id" value="{}" />
                    
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">Token to Buy</label>
                        <input type="text" name="token" placeholder="Token UUID (e.g. 1234...)" class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all" required>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">Amount</label>
                         <div class="relative">
                            <span class="absolute left-3 top-3 text-gray-500">$</span>
                            <input type="number" name="amount" placeholder="100.00" class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 pl-8 text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all" required>
                        </div>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">Email Address</label>
                        <input type="email" name="email" placeholder="you@example.com" class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all" required>
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">Result Page URL</label>
                        <input type="text" name="result_page" value="http://localhost:3000/ui" class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all">
                    </div>

                    <button type="submit" class="w-full bg-green-600 hover:bg-green-500 text-white font-bold py-4 rounded-lg shadow-lg hover:shadow-green-500/20 transition-all transform hover:-translate-y-0.5">
                        Proceed to Payment
                    </button>
                    
                    <div id="on-ramp-result" class="mt-4"></div>
                 </form>
            </div>
        </div>
        "##,
        account_id
    )
}

pub fn faucet_tab(account_id: Uuid, assets: Vec<AssetBookRecord>) -> String {
    let mut asset_opts = String::new();
    for a in assets {
        asset_opts.push_str(&format!(
            r##"<option value="{}">{} ({})</option>"##,
            a.id, a.symbol, a.id 
        ));
    }

    format!(
         r##"
        <div class="max-w-2xl mx-auto space-y-8">
            <div class="text-center">
                <h2 class="text-3xl font-bold text-white mb-2">Testnet Faucet</h2>
                <p class="text-gray-400">Request free tokens for testing purposes.</p>
            </div>

            <div class="bg-gray-800 p-8 rounded-2xl border border-gray-700 shadow-xl">
                 <form hx-post="/ui/faucet" hx-target="#faucet-result" class="space-y-6">
                    <input type="hidden" name="account_id" value="{}" />
                    
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-2">Token to Request</label>
                        <select name="asset_id" class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all" required>
                            <option value="">-- Select Token --</option>
                            {}
                        </select>
                    </div>

                    <button type="submit" class="w-full bg-purple-600 hover:bg-purple-500 text-white font-bold py-4 rounded-lg shadow-lg hover:shadow-purple-500/20 transition-all transform hover:-translate-y-0.5">
                        Request Airdrop
                    </button>
                    
                    <div id="faucet-result" class="mt-4"></div>
                 </form>
            </div>
        </div>
        "##,
        account_id,
        asset_opts
    )
}
// Lending Tab Templates

pub fn lending_tab(account_id: Uuid, pools: Vec<LendingPoolRecord>) -> String {
    let mut pool_opts = String::new();
    for p in &pools {
        let name = p.name.as_ref().map(|n| n.as_str()).unwrap_or("Unnamed Pool");
        pool_opts.push_str(&format!(
            r##"<option value="{}">{}</option>"##,
            p.id, name
        ));
    }

    format!(
        r##"
        <div class="space-y-6">
            <div class="text-center">
                <h2 class="text-3xl font-bold text-white mb-2">Asset Lending</h2>
                <p class="text-gray-400">Supply liquidity, borrow assets, and manage your positions.</p>
            </div>

            <!-- Pool Selector -->
            <div class="bg-gray-800 p-6 rounded-2xl border border-gray-700">
                <label class="block text-sm font-medium text-gray-300 mb-2">Select Pool</label>
                <select id="pool-selector" 
                        class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3 focus:ring-2 focus:ring-blue-500"
                        hx-get="/ui/lending/pool_stats"
                        hx-target="#pool-stats"
                        hx-swap="innerHTML"
                        hx-include="[name='account_id']">
                    <option value="">-- Select a Pool --</option>
                    {}
                </select>
                <input type="hidden" name="account_id" value="{}" />
            </div>

            <!-- Pool Stats Display -->
            <div id="pool-stats" class="bg-gray-800 p-6 rounded-2xl border border-gray-700">
                <p class="text-gray-400 text-center">Select a pool to view statistics</p>
            </div>

            <!-- Action Tabs -->
            <div class="bg-gray-800 rounded-2xl border border-gray-700">
                <div class="flex border-b border-gray-700">
                    <button class="px-6 py-3 text-white font-medium border-b-2 border-blue-500 lending-action-tab" data-action="supply">Supply</button>
                    <button class="px-6 py-3 text-gray-400 font-medium lending-action-tab" data-action="borrow">Borrow</button>
                    <button class="px-6 py-3 text-gray-400 font-medium lending-action-tab" data-action="withdraw">Withdraw</button>
                    <button class="px-6 py-3 text-gray-400 font-medium lending-action-tab" data-action="repay">Repay</button>
                </div>
                <div id="lending-action-content" class="p-6">
                    <!-- Supply form will be loaded here -->
                </div>
            </div>

            <!-- User Positions -->
            <div id="user-positions" class="bg-gray-800 p-6 rounded-2xl border border-gray-700">
                <h3 class="text-xl font-bold text-white mb-4">Your Positions</h3>
                <p class="text-gray-400 text-center">Select a pool to view your positions</p>
            </div>
        </div>

        <script>
            // Tab switching for lending actions
            document.querySelectorAll('.lending-action-tab').forEach(btn => {{
                btn.addEventListener('click', function() {{
                    document.querySelectorAll('.lending-action-tab').forEach(b => {{
                        b.classList.remove('border-blue-500', 'text-white');
                        b.classList.add('text-gray-400');
                    }});
                    this.classList.remove('text-gray-400');
                    this.classList.add('border-blue-500', 'text-white');
                    
                    const action = this.dataset.action;
                    const poolId = document.getElementById('pool-selector').value;
                    const accountId = '{}';//
                    
                    if (!poolId) {{
                        document.getElementById('lending-action-content').innerHTML = '<p class="text-gray-400">Please select a pool first</p>';
                        return;
                    }}
                    
                    // Load the appropriate form
                    htmx.ajax('GET', `/ui/lending/${{action}}_form?pool_id=${{poolId}}&account_id=${{accountId}}`, {{target: '#lending-action-content'}});
                }});
            }});
            
            // When pool changes, update positions
            document.getElementById('pool-selector').addEventListener('change', function() {{
                const poolId = this.value;
                const accountId = '{}';
                if (poolId) {{
                    htmx.ajax('GET', `/ui/lending/user_positions?pool_id=${{poolId}}&wallet_id=${{accountId}}`, {{target: '#user-positions'}});
                }}
            }});
        </script>
        "##,
        pool_opts, account_id, account_id, account_id
    )
}

pub fn supply_form(pool_id: Uuid, account_id: Uuid) -> String {
    format!(
        r##"
        <form hx-post="/ui/lending/supply" hx-target="#lending-result" class="space-y-4">
            <input type="hidden" name="pool_id" value="{}" />
            <input type="hidden" name="account_id" value="{}" />
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Amount to Supply</label>
                <input type="number" step="0.000001" name="amount" placeholder="0.00" 
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white focus:ring-2 focus:ring-blue-500" required>
            </div>
            
            <button type="submit" class="w-full bg-blue-600 hover:bg-blue-500 text-white font-bold py-3 rounded-lg">
                Supply Liquidity
            </button>
            
            <div id="lending-result"></div>
        </form>
        "##,
        pool_id, account_id
    )
}

pub fn borrow_form(pool_id: Uuid, account_id: Uuid, ltv: String, assets: Vec<AssetBookRecord>) -> String {
    let mut asset_opts = String::new();
    for asset in assets {
        asset_opts.push_str(&format!(
            r##"<option value="{}">{} ({})</option>"##,
            asset.id, asset.symbol, asset.name
        ));
    }
    
    format!(
        r##"
        <form hx-post="/ui/lending/borrow" hx-target="#lending-result" class="space-y-4">
            <input type="hidden" name="pool_id" value="{}" />
            <input type="hidden" name="account_id" value="{}" />
            <input type="hidden" id="ltv-value" value="{}" />
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Loan Amount</label>
                <input type="number" step="0.000001" name="loan_amount" id="loan-amount" placeholder="0.00" 
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white focus:ring-2 focus:ring-blue-500" required>
            </div>
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Collateral Asset</label>
                <select name="collateral_asset" class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3 focus:ring-2 focus:ring-blue-500" required>
                    <option value="">-- Select Collateral --</option>
                    {}
                </select>
            </div>
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Collateral Price (in loan asset)</label>
                <input type="number" step="0.000001" name="collateral_price" id="collateral-price" placeholder="0.00" 
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white focus:ring-2 focus:ring-blue-500" required>
            </div>
            
            <div class="bg-gray-900/50 p-4 rounded-lg">
                <p class="text-sm text-gray-400 mb-1">Required Collateral:</p>
                <p class="text-2xl font-bold text-blue-400" id="calculated-collateral">0.00</p>
            </div>
            
            <button type="submit" class="w-full bg-green-600 hover:bg-green-500 text-white font-bold py-3 rounded-lg">
                Borrow Assets
            </button>
            
            <div id="lending-result"></div>
        </form>
        
        <script>
            function calculateCollateral() {{
                const loanAmount = parseFloat(document.getElementById('loan-amount').value) || 0;
                const price = parseFloat(document.getElementById('collateral-price').value) || 0;
                const ltv = parseFloat(document.getElementById('ltv-value').value) || 0;
                
                if (loanAmount > 0 && price > 0 && ltv > 0) {{
                    // LTV is in basis points (7500 = 75%), so use 10000 for 100%
                    const collateral = ((10000 / ltv) * loanAmount) / price;
                    document.getElementById('calculated-collateral').textContent = collateral.toFixed(6);
                }} else {{
                    document.getElementById('calculated-collateral').textContent = '0.00';
                }}
            }}
            
            document.getElementById('loan-amount').addEventListener('input', calculateCollateral);
            document.getElementById('collateral-price').addEventListener('input', calculateCollateral);
        </script>
        "##,
        pool_id, account_id, ltv, asset_opts
    )
}

pub fn withdraw_form(pool_id: Uuid, account_id: Uuid) -> String {
    format!(
        r##"
        <form hx-post="/ui/lending/withdraw" hx-target="#lending-result" class="space-y-4">
            <input type="hidden" name="pool_id" value="{}" />
            <input type="hidden" name="account_id" value="{}" />
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Yield Token Amount</label>
                <input type="number" step="0.000001" name="amount" placeholder="0.00" 
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white focus:ring-2 focus:ring-blue-500" required>
            </div>
            
            <button type="submit" class="w-full bg-yellow-600 hover:bg-yellow-500 text-white font-bold py-3 rounded-lg">
                Withdraw Liquidity
            </button>
            
            <div id="lending-result"></div>
        </form>
        "##,
        pool_id, account_id
    )
}

pub fn repay_form(account_id: Uuid, loans: Vec<LoanRecord>) -> String {
    let mut loan_opts = String::new();
    for loan in loans {
        loan_opts.push_str(&format!(
            r##"<option value="{}">Loan {} - Pool: {}</option>"##,
            loan.id, loan.id, loan.pool
        ));
    }

    format!(
        r##"
        <form hx-post="/ui/lending/repay" hx-target="#lending-result" class="space-y-4">
            <input type="hidden" name="account_id" value="{}" />
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Select Loan</label>
                <select name="loan_id" class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3 focus:ring-2 focus:ring-blue-500" required>
                    <option value="">-- Select a Loan --</option>
                    {}
                </select>
            </div>
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Repayment Amount</label>
                <input type="number" step="0.000001" name="amount" placeholder="0.00" 
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white focus:ring-2 focus:ring-blue-500" required>
            </div>
            
            <button type="submit" class="w-full bg-purple-600 hover:bg-purple-500 text-white font-bold py-3 rounded-lg">
                Repay Loan
            </button>
            
            <div id="lending-result"></div>
        </form>
        "##,
        account_id, loan_opts
    )
}
// Listing Tab Templates

pub fn listings_tab(account_id: Uuid, listings: Vec<CradleNativeListingRow>, companies: Vec<CompanyRow>) -> String {
    let mut listing_opts = String::new();
    for l in &listings {
        listing_opts.push_str(&format!(
            r##"<option value="{}">{} - {}</option>"##,
            l.id, l.name, format!("{:?}", l.status)
        ));
    }

    format!(
        r##"
        <div class="space-y-6">
            <div class="text-center">
                <h2 class="text-3xl font-bold text-white mb-2">Native Listings</h2>
                <p class="text-gray-400">Create and manage asset listings (IPO-like functionality).</p>
            </div>

            <!-- Action Buttons -->
            <div class="flex gap-4">
                <button hx-get="/ui/listings/create_company_form?account_id={}" hx-target="#listing-form-area" 
                        class="bg-purple-600 hover:bg-purple-500 px-4 py-2 rounded text-white font-bold">
                    + Create Company
                </button>
                <button hx-get="/ui/listings/create_listing_form?account_id={}" hx-target="#listing-form-area"
                        class="bg-blue-600 hover:bg-blue-500 px-4 py-2 rounded text-white font-bold">
                    + Create Listing
                </button>
            </div>

            <!-- Form Area -->
            <div id="listing-form-area" class="bg-gray-800 p-6 rounded-2xl border border-gray-700">
                <p class="text-gray-400 text-center">Select an action above</p>
            </div>

            <!-- Listing Selector -->
            <div class="bg-gray-800 p-6 rounded-2xl border border-gray-700">
                <label class="block text-sm font-medium text-gray-300 mb-2">Select Listing</label>
                <select id="listing-selector" 
                        class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3 focus:ring-2 focus:ring-blue-500"
                        hx-get="/ui/listings/stats"
                        hx-target="#listing-stats"
                        hx-swap="innerHTML"
                        hx-include="[name='account_id']">
                    <option value="">-- Select a Listing --</option>
                    {}
                </select>
                <input type="hidden" name="account_id" value="{}" />
            </div>

            <!-- Listing Stats -->
            <div id="listing-stats" class="bg-gray-800 p-6 rounded-2xl border border-gray-700">
                <p class="text-gray-400 text-center">Select a listing to view details</p>
            </div>

            <!-- Action Tabs -->
            <div class="bg-gray-800 rounded-2xl border border-gray-700">
                <div class="flex border-b border-gray-700">
                    <button class="px-6 py-3 text-white font-medium border-b-2 border-blue-500 listing-action-tab" data-action="purchase">Purchase</button>
                    <button class="px-6 py-3 text-gray-400 font-medium listing-action-tab" data-action="return">Return</button>
                    <button class="px-6 py-3 text-gray-400 font-medium listing-action-tab" data-action="withdraw">Withdraw</button>
                </div>
                <div id="listing-action-content" class="p-6">
                    <p class="text-gray-400">Select a listing first</p>
                </div>
            </div>
        </div>

        <script>
            document.querySelectorAll('.listing-action-tab').forEach(btn => {{
                btn.addEventListener('click', function() {{
                    document.querySelectorAll('.listing-action-tab').forEach(b => {{
                        b.classList.remove('border-blue-500', 'text-white');
                        b.classList.add('text-gray-400');
                    }});
                    this.classList.remove('text-gray-400');
                    this.classList.add('border-blue-500', 'text-white');
                    
                    const action = this.dataset.action;
                    const listingId = document.getElementById('listing-selector').value;
                    const accountId = '{}';
                    
                    if (!listingId) {{
                        document.getElementById('listing-action-content').innerHTML = '<p class="text-gray-400">Please select a listing first</p>';
                        return;
                    }}
                    
                    htmx.ajax('GET', `/ui/listings/${{action}}_form?listing_id=${{listingId}}&account_id=${{accountId}}`, {{target: '#listing-action-content'}});
                }});
            }});
        </script>
        "##,
        account_id, account_id, listing_opts, account_id, account_id
    )
}

pub fn create_company_form(account_id: Uuid) -> String {
    format!(
        r##"
        <h3 class="text-xl font-bold text-white mb-4">Create Company</h3>
        <form hx-post="/ui/listings/create_company" hx-target="#listing-result" class="space-y-4">
            <input type="hidden" name="account_id" value="{}" />
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Company Name</label>
                <input type="text" name="name" class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required>
            </div>
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Description</label>
                <textarea name="description" rows="3" class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required></textarea>
            </div>
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Legal Documents</label>
                <input type="text" name="legal_documents" placeholder="Document URL/Hash" class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required>
            </div>
            
            <button type="submit" class="w-full bg-purple-600 hover:bg-purple-500 text-white font-bold py-3 rounded-lg">
                Create Company
            </button>
            
            <div id="listing-result"></div>
        </form>
        "##,
        account_id
    )
}

pub fn create_listing_form(account_id: Uuid, companies: Vec<CompanyRow>, assets: Vec<AssetBookRecord>) -> String {
    let mut company_opts = String::new();
    for c in companies {
        company_opts.push_str(&format!(r##"<option value="{}">{}</option>"##, c.id, c.name));
    }
    
    let mut asset_opts = String::new();
    for a in assets {
        asset_opts.push_str(&format!(r##"<option value="{}">{} ({})</option>"##, a.id, a.symbol, a.name));
    }

    format!(
        r##"
        <h3 class="text-xl font-bold text-white mb-4">Create Listing</h3>
        <form hx-post="/ui/listings/create_listing" hx-target="#listing-result" class="space-y-4">
            <input type="hidden" name="account_id" value="{}" />
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Company</label>
                <select name="company" class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3" required>
                    <option value="">-- Select Company --</option>
                    {}
                </select>
            </div>
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Name</label>
                <input type="text" name="name" class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required>
            </div>
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Description</label>
                <textarea name="description" rows="2" class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required></textarea>
            </div>
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Listed Asset</label>
                <select name="listed_asset" class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3" required>
                    <option value="">-- Select Asset --</option>
                    {}
                </select>
            </div>
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Purchase With Asset</label>
                <select name="purchase_asset" class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3" required>
                    <option value="">-- Select Asset --</option>
                    {}
                </select>
            </div>
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Purchase Price</label>
                <input type="number" step="0.000001" name="purchase_price" class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required>
            </div>
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Max Supply</label>
                <input type="number" step="0.000001" name="max_supply" class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required>
            </div>
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Documents</label>
                <input type="text" name="documents" placeholder="Document URL/Hash" class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required>
            </div>
            
            <button type="submit" class="w-full bg-blue-600 hover:bg-blue-500 text-white font-bold py-3 rounded-lg">
                Create Listing
            </button>
            
            <div id="listing-result"></div>
        </form>
        "##,
        account_id, company_opts, asset_opts, asset_opts
    )
}

pub fn purchase_listing_form(listing_id: Uuid, account_id: Uuid) -> String {
    format!(
        r##"
        <form hx-post="/ui/listings/purchase" hx-target="#listing-action-result" class="space-y-4">
            <input type="hidden" name="listing_id" value="{}" />
            <input type="hidden" name="account_id" value="{}" />
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Amount to Purchase</label>
                <input type="number" step="0.000001" name="amount" placeholder="0.00" 
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required>
            </div>
            
            <button type="submit" class="w-full bg-green-600 hover:bg-green-500 text-white font-bold py-3 rounded-lg">
                Purchase Assets
            </button>
            
            <div id="listing-action-result"></div>
        </form>
        "##,
        listing_id, account_id
    )
}

pub fn return_listing_form(listing_id: Uuid, account_id: Uuid)  -> String {
    format!(
        r##"
        <form hx-post="/ui/listings/return" hx-target="#listing-action-result" class="space-y-4">
            <input type="hidden" name="listing_id" value="{}" />
            <input type="hidden" name="account_id" value="{}" />
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Amount to Return</label>
                <input type="number" step="0.000001" name="amount" placeholder="0.00" 
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required>
            </div>
            
            <button type="submit" class="w-full bg-red-600 hover:bg-red-500 text-white font-bold py-3 rounded-lg">
                Return Assets
            </button>
            
            <div id="listing-action-result"></div>
        </form>
        "##,
        listing_id, account_id
    )
}

pub fn withdraw_listing_form(listing_id: Uuid, account_id: Uuid) -> String {
    format!(
        r##"
        <form hx-post="/ui/listings/withdraw" hx-target="#listing-action-result" class="space-y-4">
            <input type="hidden" name="listing_id" value="{}" />
            <input type="hidden" name="account_id" value="{}" />
            
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Amount to Withdraw</label>
                <input type="number" step="0.000001" name="amount" placeholder="0.00" 
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required>
            </div>
            
            <button type="submit" class="w-full bg-yellow-600 hover:bg-yellow-500 text-white font-bold py-3 rounded-lg">
                Withdraw to Beneficiary
            </button>
            
            <div id="listing-action-result"></div>
        </form>
        "##,
        listing_id, account_id
    )
}
// Oracle Tab Templates

pub fn oracle_tab(account_id: Uuid, pools: Vec<LendingPoolRecord>, assets: Vec<AssetBookRecord>) -> String {
    let mut pool_opts = String::new();
    for p in &pools {
        let name = p.name.as_ref().map(|n| n.as_str()).unwrap_or("Unnamed Pool");
        pool_opts.push_str(&format!(
            r##"<option value="{}">{}</option>"##,
            p.id, name
        ));
    }

    let mut asset_opts = String::new();
    for a in &assets {
        asset_opts.push_str(&format!(
            r##"<option value="{}">{} ({})</option>"##,
            a.id, a.symbol, a.name
        ));
    }

    format!(
        r##"
        <div class="space-y-6">
            <div class="text-center">
                <h2 class="text-3xl font-bold text-white mb-2">Oracle Price Configuration</h2>
                <p class="text-gray-400">Configure oracle prices for lending pool assets.</p>
            </div>

            <!-- Pool Selector -->
            <div class="bg-gray-800 p-6 rounded-2xl border border-gray-700">
                <label class="block text-sm font-medium text-gray-300 mb-2">Select Lending Pool</label>
                <select id="oracle-pool-selector"
                        class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3 focus:ring-2 focus:ring-blue-500">
                    <option value="">-- Select a Pool --</option>
                    {}
                </select>
            </div>

            <!-- Asset Selector -->
            <div class="bg-gray-800 p-6 rounded-2xl border border-gray-700">
                <label class="block text-sm font-medium text-gray-300 mb-2">Select Asset</label>
                <select id="oracle-asset-selector"
                        class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3 focus:ring-2 focus:ring-blue-500">
                    <option value="">-- Select an Asset --</option>
                    {}
                </select>
            </div>

            <!-- Price Configuration Form -->
            <div class="bg-gray-800 p-6 rounded-2xl border border-gray-700">
                <h3 class="text-xl font-bold text-white mb-4">Set Oracle Price</h3>
                <div id="oracle-form-content">
                    <p class="text-gray-400 text-center">Select a pool and asset to configure pricing</p>
                </div>
            </div>
        </div>

        <script>
            const poolSelector = document.getElementById('oracle-pool-selector');
            const assetSelector = document.getElementById('oracle-asset-selector');
            const formContent = document.getElementById('oracle-form-content');
            const accountId = '{}';

            function updateForm() {{
                const poolId = poolSelector.value;
                const assetId = assetSelector.value;

                if (!poolId || !assetId) {{
                    formContent.innerHTML = '<p class="text-gray-400 text-center">Select a pool and asset to configure pricing</p>';
                    return;
                }}

                // Render the price form
                formContent.innerHTML = `
                    <form hx-post="/ui/oracle/set_price" hx-target="#oracle-result" class="space-y-4">
                        <input type="hidden" name="pool_id" value="${{poolId}}" />
                        <input type="hidden" name="asset_id" value="${{assetId}}" />

                        <div>
                            <label class="block text-sm font-medium text-gray-300 mb-2">Price Multiplier</label>
                            <input type="number" step="0.000001" name="price" placeholder="1.0"
                                   class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white focus:ring-2 focus:ring-blue-500" required>
                            <p class="text-xs text-gray-500 mt-1">Enter price in reserve asset decimals (e.g., 1.5 for 1.5x)</p>
                        </div>

                        <button type="submit" class="w-full bg-blue-600 hover:bg-blue-500 text-white font-bold py-3 rounded-lg">
                            Update Oracle Price
                        </button>

                        <div id="oracle-result"></div>
                    </form>
                `;

                // Re-process htmx for the new form
                htmx.process(formContent);
            }}

            poolSelector.addEventListener('change', updateForm);
            assetSelector.addEventListener('change', updateForm);
        </script>
        "##,
        pool_opts, asset_opts, account_id
    )
}

// ============================================================================
// GLOBAL ADMIN TOOLS TEMPLATES (No account selection required)
// ============================================================================

pub fn admin_tools_page() -> String {
    base_layout(
        r##"
        <div class="flex-1 flex flex-col min-w-0 bg-gray-900">
            <!-- Top Bar -->
            <div class="bg-gray-800 border-b border-gray-700 px-6 py-4 flex items-center justify-between shadow-md z-10">
                <div>
                    <div class="text-xs text-gray-500 uppercase tracking-wider font-semibold">Global Admin Tools</div>
                    <div class="text-xl font-mono text-white">System Management</div>
                </div>
                <a href="/" class="text-blue-400 hover:text-blue-300 text-sm">
                    &larr; Back to Account Dashboard
                </a>
            </div>

            <!-- Tabs Navigation -->
            <div class="flex border-b border-gray-700 bg-gray-800/50 px-6 pt-2 gap-1">
                <button class="px-6 py-3 text-sm font-medium text-blue-400 border-b-2 border-blue-400 hover:bg-gray-700/50 rounded-t-lg transition-colors focus:outline-none admin-tab"
                        hx-get="/ui/admin/tabs/assets"
                        hx-target="#admin-tab-content">
                    Assets/Tokens
                </button>
                <button class="px-6 py-3 text-sm font-medium text-gray-400 border-b-2 border-transparent hover:text-gray-200 hover:bg-gray-700/50 rounded-t-lg transition-colors focus:outline-none admin-tab"
                        hx-get="/ui/admin/tabs/markets"
                        hx-target="#admin-tab-content">
                    Markets
                </button>
                <button class="px-6 py-3 text-sm font-medium text-gray-400 border-b-2 border-transparent hover:text-gray-200 hover:bg-gray-700/50 rounded-t-lg transition-colors focus:outline-none admin-tab"
                        hx-get="/ui/admin/tabs/aggregator"
                        hx-target="#admin-tab-content">
                    Time Series Aggregator
                </button>
                <button class="px-6 py-3 text-sm font-medium text-gray-400 border-b-2 border-transparent hover:text-gray-200 hover:bg-gray-700/50 rounded-t-lg transition-colors focus:outline-none admin-tab"
                        hx-get="/ui/admin/tabs/accounts"
                        hx-target="#admin-tab-content">
                    Accounts/KYC
                </button>
            </div>

            <!-- Tab Content Area -->
            <div id="admin-tab-content" class="flex-1 overflow-y-auto p-6" hx-get="/ui/admin/tabs/assets" hx-trigger="load">
                <div class="flex justify-center items-center h-full text-gray-500 animate-pulse">Loading...</div>
            </div>
        </div>

        <script>
            document.querySelectorAll('.admin-tab').forEach(btn => {
                btn.addEventListener('click', function() {
                    document.querySelectorAll('.admin-tab').forEach(b => {
                        b.classList.remove('border-blue-400', 'text-blue-400');
                        b.classList.add('border-transparent', 'text-gray-400');
                    });
                    this.classList.remove('border-transparent', 'text-gray-400');
                    this.classList.add('border-blue-400', 'text-blue-400');
                });
            });
        </script>
        "##
    )
}

pub fn admin_assets_tab(assets: Vec<AssetBookRecord>) -> String {
    let mut assets_html = String::new();
    for a in &assets {
        assets_html.push_str(&format!(
            r##"
            <tr class="border-b border-gray-700 hover:bg-gray-700/50">
                <td class="px-4 py-3 font-mono text-sm text-blue-400">{}</td>
                <td class="px-4 py-3 text-sm font-bold text-white">{}</td>
                <td class="px-4 py-3 text-sm text-gray-300">{}</td>
                <td class="px-4 py-3 text-sm text-gray-400">{}</td>
                <td class="px-4 py-3 text-xs text-gray-500 font-mono">{}</td>
            </tr>
            "##,
            &a.symbol, &a.name, format!("{:?}", a.asset_type), a.decimals, &a.id
        ));
    }

    if assets_html.is_empty() {
        assets_html = r##"<tr><td colspan="5" class="p-4 text-center text-gray-500 italic">No assets found</td></tr>"##.to_string();
    }

    format!(
        r##"
        <div class="space-y-6">
            <div class="text-center">
                <h2 class="text-3xl font-bold text-white mb-2">Asset Management</h2>
                <p class="text-gray-400">Create new tokens or register existing ones.</p>
            </div>

            <!-- Action Buttons -->
            <div class="flex gap-4">
                <button hx-get="/ui/admin/assets/create_new_form" hx-target="#asset-form-area"
                        class="bg-green-600 hover:bg-green-500 px-4 py-2 rounded text-white font-bold">
                    + Create New Token
                </button>
                <button hx-get="/ui/admin/assets/create_existing_form" hx-target="#asset-form-area"
                        class="bg-blue-600 hover:bg-blue-500 px-4 py-2 rounded text-white font-bold">
                    + Register Existing Token
                </button>
            </div>

            <!-- Form Area -->
            <div id="asset-form-area" class="bg-gray-800 p-6 rounded-2xl border border-gray-700">
                <p class="text-gray-400 text-center">Select an action above to create or register a token</p>
            </div>

            <!-- Assets List -->
            <div class="bg-gray-800 rounded-xl border border-gray-700 overflow-hidden">
                <div class="p-4 border-b border-gray-700 bg-gray-700/30">
                    <h4 class="font-bold text-gray-200">Registered Assets ({} total)</h4>
                </div>
                <div class="overflow-x-auto">
                    <table class="w-full text-left">
                        <thead class="bg-gray-700/50 text-xs text-gray-400 uppercase">
                            <tr>
                                <th class="px-4 py-2">Symbol</th>
                                <th class="px-4 py-2">Name</th>
                                <th class="px-4 py-2">Type</th>
                                <th class="px-4 py-2">Decimals</th>
                                <th class="px-4 py-2">ID</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-700/50">
                            {}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
        "##,
        assets.len(), assets_html
    )
}

pub fn create_new_asset_form() -> String {
    r##"
    <h3 class="text-xl font-bold text-white mb-4">Create New Token</h3>
    <p class="text-gray-400 text-sm mb-4">This will deploy a new token contract on the blockchain.</p>
    <form hx-post="/ui/admin/assets/create_new" hx-target="#asset-result" class="space-y-4">
        <div class="grid grid-cols-2 gap-4">
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Token Name</label>
                <input type="text" name="name" placeholder="e.g., USD Coin"
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required>
            </div>
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Symbol</label>
                <input type="text" name="symbol" placeholder="e.g., USDC"
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required>
            </div>
        </div>

        <div class="grid grid-cols-2 gap-4">
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Decimals</label>
                <input type="number" name="decimals" value="8" min="0" max="18"
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required>
            </div>
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Asset Type</label>
                <select name="asset_type" class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3" required>
                    <option value="native">Native</option>
                    <option value="bridged">Bridged</option>
                    <option value="yield_bearing">Yield Bearing</option>
                    <option value="chain_native">Chain Native</option>
                    <option value="stablecoin">StableCoin</option>
                    <option value="volatile">Volatile</option>
                </select>
            </div>
        </div>

        <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">Icon URL (optional)</label>
            <input type="text" name="icon" placeholder="https://..."
                   class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white">
        </div>

        <button type="submit" class="w-full bg-green-600 hover:bg-green-500 text-white font-bold py-3 rounded-lg">
            Create Token
        </button>

        <div id="asset-result"></div>
    </form>
    "##.to_string()
}

pub fn create_existing_asset_form() -> String {
    r##"
    <h3 class="text-xl font-bold text-white mb-4">Register Existing Token</h3>
    <p class="text-gray-400 text-sm mb-4">Register an existing token that's already deployed on the blockchain.</p>
    <form hx-post="/ui/admin/assets/create_existing" hx-target="#asset-result" class="space-y-4">
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">Token Address</label>
            <input type="text" name="token" placeholder="0x... (Solidity address)"
                   class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white font-mono" required>
        </div>

        <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">Asset Manager Address (optional)</label>
            <input type="text" name="asset_manager" placeholder="0x... (if applicable)"
                   class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white font-mono">
        </div>

        <div class="grid grid-cols-2 gap-4">
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Token Name</label>
                <input type="text" name="name" placeholder="e.g., USD Coin"
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required>
            </div>
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Symbol</label>
                <input type="text" name="symbol" placeholder="e.g., USDC"
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required>
            </div>
        </div>

        <div class="grid grid-cols-2 gap-4">
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Decimals</label>
                <input type="number" name="decimals" value="8" min="0" max="18"
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required>
            </div>
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Asset Type</label>
                <select name="asset_type" class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3" required>
                    <option value="native">Native</option>
                    <option value="bridged">Bridged</option>
                    <option value="yield_bearing">Yield Bearing</option>
                    <option value="chain_native">Chain Native</option>
                    <option value="stablecoin">StableCoin</option>
                    <option value="volatile">Volatile</option>
                </select>
            </div>
        </div>

        <div>
            <label class="block text-sm font-medium text-gray-300 mb-2">Icon URL (optional)</label>
            <input type="text" name="icon" placeholder="https://..."
                   class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white">
        </div>

        <button type="submit" class="w-full bg-blue-600 hover:bg-blue-500 text-white font-bold py-3 rounded-lg">
            Register Token
        </button>

        <div id="asset-result"></div>
    </form>
    "##.to_string()
}

pub fn admin_markets_tab(markets: Vec<MarketRecord>, assets: Vec<AssetBookRecord>) -> String {
    let mut markets_html = String::new();

    // Create a hashmap for quick asset lookup
    let asset_map: std::collections::HashMap<Uuid, &AssetBookRecord> = assets.iter().map(|a| (a.id, a)).collect();

    for m in &markets {
        let asset_one_symbol = asset_map.get(&m.asset_one).map(|a| a.symbol.as_str()).unwrap_or("Unknown");
        let asset_two_symbol = asset_map.get(&m.asset_two).map(|a| a.symbol.as_str()).unwrap_or("Unknown");

        let status_color = match m.market_status {
            MarketStatus::Active => "text-green-400",
            MarketStatus::InActive => "text-gray-400",
            MarketStatus::Suspended => "text-red-400",
        };

        markets_html.push_str(&format!(
            r##"
            <tr class="border-b border-gray-700 hover:bg-gray-700/50">
                <td class="px-4 py-3 font-bold text-white">{}</td>
                <td class="px-4 py-3 text-sm text-blue-400">{}/{}</td>
                <td class="px-4 py-3 text-sm text-gray-300">{:?}</td>
                <td class="px-4 py-3 text-sm {}"><span class="px-2 py-1 rounded bg-gray-700">{:?}</span></td>
                <td class="px-4 py-3 text-sm text-gray-400">{:?}</td>
                <td class="px-4 py-3 text-xs text-gray-500 font-mono">{}</td>
            </tr>
            "##,
            &m.name, asset_one_symbol, asset_two_symbol, m.market_type, status_color, m.market_status, m.market_regulation, &m.id
        ));
    }

    if markets_html.is_empty() {
        markets_html = r##"<tr><td colspan="6" class="p-4 text-center text-gray-500 italic">No markets found</td></tr>"##.to_string();
    }

    // Build asset options for the create form
    let mut asset_opts = String::new();
    for a in &assets {
        asset_opts.push_str(&format!(
            r##"<option value="{}">{} ({})</option>"##,
            a.id, a.symbol, a.name
        ));
    }

    format!(
        r##"
        <div class="space-y-6">
            <div class="text-center">
                <h2 class="text-3xl font-bold text-white mb-2">Market Management</h2>
                <p class="text-gray-400">Create and manage trading markets.</p>
            </div>

            <!-- Action Button -->
            <div class="flex gap-4">
                <button hx-get="/ui/admin/markets/create_form" hx-target="#market-form-area"
                        class="bg-green-600 hover:bg-green-500 px-4 py-2 rounded text-white font-bold">
                    + Create New Market
                </button>
            </div>

            <!-- Form Area -->
            <div id="market-form-area" class="bg-gray-800 p-6 rounded-2xl border border-gray-700">
                <p class="text-gray-400 text-center">Click above to create a new market</p>
            </div>

            <!-- Markets List -->
            <div class="bg-gray-800 rounded-xl border border-gray-700 overflow-hidden">
                <div class="p-4 border-b border-gray-700 bg-gray-700/30">
                    <h4 class="font-bold text-gray-200">Active Markets ({} total)</h4>
                </div>
                <div class="overflow-x-auto">
                    <table class="w-full text-left">
                        <thead class="bg-gray-700/50 text-xs text-gray-400 uppercase">
                            <tr>
                                <th class="px-4 py-2">Name</th>
                                <th class="px-4 py-2">Pair</th>
                                <th class="px-4 py-2">Type</th>
                                <th class="px-4 py-2">Status</th>
                                <th class="px-4 py-2">Regulation</th>
                                <th class="px-4 py-2">ID</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-700/50">
                            {}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
        "##,
        markets.len(), markets_html
    )
}

pub fn create_market_form(assets: Vec<AssetBookRecord>) -> String {
    let mut asset_opts = String::new();
    for a in &assets {
        asset_opts.push_str(&format!(
            r##"<option value="{}">{} ({})</option>"##,
            a.id, a.symbol, a.name
        ));
    }

    format!(
        r##"
        <h3 class="text-xl font-bold text-white mb-4">Create New Market</h3>
        <form hx-post="/ui/admin/markets/create" hx-target="#market-result" class="space-y-4">
            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Market Name</label>
                <input type="text" name="name" placeholder="e.g., BTC/USDC Spot"
                       class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white" required>
            </div>

            <div>
                <label class="block text-sm font-medium text-gray-300 mb-2">Description (optional)</label>
                <textarea name="description" rows="2" placeholder="Market description..."
                          class="w-full bg-gray-900 border border-gray-600 rounded-lg p-3 text-white"></textarea>
            </div>

            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label class="block text-sm font-medium text-gray-300 mb-2">Base Asset (Asset One)</label>
                    <select name="asset_one" class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3" required>
                        <option value="">-- Select Base Asset --</option>
                        {}
                    </select>
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-300 mb-2">Quote Asset (Asset Two)</label>
                    <select name="asset_two" class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3" required>
                        <option value="">-- Select Quote Asset --</option>
                        {}
                    </select>
                </div>
            </div>

            <div class="grid grid-cols-2 gap-4">
                <div>
                    <label class="block text-sm font-medium text-gray-300 mb-2">Market Type</label>
                    <select name="market_type" class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3" required>
                        <option value="spot">Spot</option>
                        <option value="derivative">Derivative</option>
                        <option value="futures">Futures</option>
                    </select>
                </div>
                <div>
                    <label class="block text-sm font-medium text-gray-300 mb-2">Regulation</label>
                    <select name="market_regulation" class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3" required>
                        <option value="unregulated">Unregulated</option>
                        <option value="regulated">Regulated</option>
                    </select>
                </div>
            </div>

            <button type="submit" class="w-full bg-green-600 hover:bg-green-500 text-white font-bold py-3 rounded-lg">
                Create Market
            </button>

            <div id="market-result"></div>
        </form>
        "##,
        asset_opts, asset_opts
    )
}

pub fn admin_aggregator_tab(markets: Vec<MarketRecord>, assets: Vec<AssetBookRecord>) -> String {
    let mut market_opts = String::new();

    // Create a hashmap for quick asset lookup
    let asset_map: std::collections::HashMap<Uuid, &AssetBookRecord> = assets.iter().map(|a| (a.id, a)).collect();

    for m in &markets {
        let asset_one_symbol = asset_map.get(&m.asset_one).map(|a| a.symbol.as_str()).unwrap_or("Unknown");
        let asset_two_symbol = asset_map.get(&m.asset_two).map(|a| a.symbol.as_str()).unwrap_or("Unknown");

        market_opts.push_str(&format!(
            r##"<option value="{}" data-asset-one="{}" data-asset-two="{}" data-asset-one-symbol="{}" data-asset-two-symbol="{}">{} ({}/{})</option>"##,
            m.id, m.asset_one, m.asset_two, asset_one_symbol, asset_two_symbol, m.name, asset_one_symbol, asset_two_symbol
        ));
    }

    format!(
        r##"
        <div class="space-y-6">
            <div class="text-center">
                <h2 class="text-3xl font-bold text-white mb-2">Time Series Aggregator</h2>
                <p class="text-gray-400">Generate OHLC data from orderbook trades.</p>
            </div>

            <!-- Mode Toggle -->
            <div class="flex gap-2 justify-center">
                <button type="button" onclick="toggleAggregatorMode('manual')" id="agg-mode-manual"
                        class="px-6 py-2 rounded-lg bg-purple-600 text-white font-medium transition-colors">
                    Manual Mode
                </button>
                <button type="button" onclick="toggleAggregatorMode('batch')" id="agg-mode-batch"
                        class="px-6 py-2 rounded-lg bg-gray-700 text-gray-300 font-medium transition-colors hover:bg-gray-600">
                    Batch Mode
                </button>
            </div>

            <!-- Manual Mode Form -->
            <div id="manual-mode-section" class="bg-gray-800 p-6 rounded-2xl border border-gray-700">
                <h3 class="text-lg font-bold text-white mb-4">Manual Aggregation</h3>
                <form id="manual-form" class="space-y-4">
                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-300 mb-2">Select Market</label>
                            <select id="aggregator-market" name="market_id" class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3" required>
                                <option value="">-- Select Market --</option>
                                {market_opts}
                            </select>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-300 mb-2">Select Asset</label>
                            <select id="aggregator-asset" name="asset_id" class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3" required>
                                <option value="">-- Select Market First --</option>
                            </select>
                        </div>
                    </div>

                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-300 mb-2">Aggregation Interval</label>
                            <select name="interval" class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3" required>
                                <option value="15secs">15 Seconds</option>
                                <option value="30secs">30 Seconds</option>
                                <option value="45secs">45 Seconds</option>
                                <option value="1min">1 Minute</option>
                                <option value="5min">5 Minutes</option>
                                <option value="15min" selected>15 Minutes</option>
                                <option value="30min">30 Minutes</option>
                                <option value="1hr">1 Hour</option>
                                <option value="4hr">4 Hours</option>
                                <option value="1day">1 Day</option>
                                <option value="1week">1 Week</option>
                            </select>
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-gray-300 mb-2">Time Range</label>
                            <select name="duration" class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3" required>
                                <option value="24h">Last 24 Hours</option>
                                <option value="7d" selected>Last 7 Days</option>
                                <option value="30d">Last 30 Days</option>
                                <option value="90d">Last 90 Days</option>
                                <option value="all">All Time</option>
                            </select>
                        </div>
                    </div>

                    <button type="button" onclick="runManualAggregation()" id="manual-run-btn"
                            class="w-full bg-purple-600 hover:bg-purple-500 text-white font-bold py-3 rounded-lg transition-colors">
                        Run Aggregation
                    </button>
                </form>

                <div id="manual-result" class="mt-4"></div>
            </div>

            <!-- Batch Mode Form -->
            <div id="batch-mode-section" class="hidden bg-gray-800 p-6 rounded-2xl border border-gray-700">
                <h3 class="text-lg font-bold text-white mb-4">Batch Aggregation</h3>
                <p class="text-gray-400 text-sm mb-4">
                    Run all standard aggregations for a market automatically:
                </p>

                <div class="grid grid-cols-3 gap-4 mb-4 text-sm">
                    <div class="bg-gray-700/50 p-3 rounded-lg">
                        <div class="font-bold text-purple-400 mb-1">24 Hours</div>
                        <div class="text-gray-400">15s, 30s, 45s, 1m, 15m, 30m, 1h, 4h</div>
                    </div>
                    <div class="bg-gray-700/50 p-3 rounded-lg">
                        <div class="font-bold text-blue-400 mb-1">7 Days</div>
                        <div class="text-gray-400">1 day intervals</div>
                    </div>
                    <div class="bg-gray-700/50 p-3 rounded-lg">
                        <div class="font-bold text-green-400 mb-1">30 Days</div>
                        <div class="text-gray-400">1 week intervals</div>
                    </div>
                </div>

                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-300 mb-2">Select Market</label>
                    <select id="batch-market" class="w-full bg-gray-900 border border-gray-600 text-gray-100 rounded-lg p-3" required>
                        <option value="">-- Select Market --</option>
                        {market_opts}
                    </select>
                </div>

                <div class="bg-yellow-900/30 border border-yellow-700 rounded-lg p-3 mb-4">
                    <div class="text-yellow-400 text-sm">
                        <strong>Note:</strong> This will process both assets in the market and may take several minutes depending on trade volume.
                    </div>
                </div>

                <button type="button" onclick="runBatchAggregation()" id="batch-run-btn"
                        class="w-full bg-gradient-to-r from-purple-600 to-blue-600 hover:from-purple-500 hover:to-blue-500 text-white font-bold py-3 rounded-lg transition-colors">
                    <span id="batch-btn-text">Run Batch Aggregation</span>
                    <span id="batch-btn-loading" class="hidden">
                        <svg class="animate-spin inline-block w-5 h-5 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                        Processing... This may take a few minutes
                    </span>
                </button>

                <div id="batch-result" class="mt-4"></div>
            </div>

            <!-- Info Box -->
            <div class="bg-gray-800/50 p-6 rounded-2xl border border-gray-700">
                <h4 class="font-bold text-gray-200 mb-2">How it works</h4>
                <ul class="text-sm text-gray-400 space-y-2">
                    <li><strong>Manual Mode:</strong> Select specific market, asset, interval and time range</li>
                    <li><strong>Batch Mode:</strong> Automatically runs all standard intervals for both assets in a market</li>
                    <li>OHLC data is generated from settled orderbook trades</li>
                </ul>
            </div>
        </div>

        <script>
            let aggregatorMode = 'manual';

            function toggleAggregatorMode(mode) {{
                aggregatorMode = mode;
                const manualBtn = document.getElementById('agg-mode-manual');
                const batchBtn = document.getElementById('agg-mode-batch');
                const manualSection = document.getElementById('manual-mode-section');
                const batchSection = document.getElementById('batch-mode-section');

                if (mode === 'manual') {{
                    manualBtn.className = 'px-6 py-2 rounded-lg bg-purple-600 text-white font-medium transition-colors';
                    batchBtn.className = 'px-6 py-2 rounded-lg bg-gray-700 text-gray-300 font-medium transition-colors hover:bg-gray-600';
                    manualSection.classList.remove('hidden');
                    batchSection.classList.add('hidden');
                }} else {{
                    batchBtn.className = 'px-6 py-2 rounded-lg bg-purple-600 text-white font-medium transition-colors';
                    manualBtn.className = 'px-6 py-2 rounded-lg bg-gray-700 text-gray-300 font-medium transition-colors hover:bg-gray-600';
                    batchSection.classList.remove('hidden');
                    manualSection.classList.add('hidden');
                }}
            }}

            document.getElementById('aggregator-market').addEventListener('change', function() {{
                const select = this;
                const assetSelect = document.getElementById('aggregator-asset');
                const selectedOption = select.options[select.selectedIndex];

                if (!selectedOption.value) {{
                    assetSelect.innerHTML = '<option value="">-- Select Market First --</option>';
                    return;
                }}

                const assetOne = selectedOption.dataset.assetOne;
                const assetTwo = selectedOption.dataset.assetTwo;
                const assetOneSymbol = selectedOption.dataset.assetOneSymbol;
                const assetTwoSymbol = selectedOption.dataset.assetTwoSymbol;

                assetSelect.innerHTML = `
                    <option value="">-- Select Asset --</option>
                    <option value="${{assetOne}}">${{assetOneSymbol}} (Base)</option>
                    <option value="${{assetTwo}}">${{assetTwoSymbol}} (Quote)</option>
                `;
            }});

            function runManualAggregation() {{
                const form = document.getElementById('manual-form');
                const marketId = form.querySelector('[name="market_id"]').value;
                const assetId = form.querySelector('[name="asset_id"]').value;
                const interval = form.querySelector('[name="interval"]').value;
                const duration = form.querySelector('[name="duration"]').value;

                if (!marketId || !assetId) {{
                    document.getElementById('manual-result').innerHTML =
                        "<div class='text-red-400'>Please select a market and asset</div>";
                    return;
                }}

                const btn = document.getElementById('manual-run-btn');
                btn.disabled = true;
                btn.innerHTML = '<svg class="animate-spin inline-block w-5 h-5 mr-2" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path></svg> Processing...';

                document.getElementById('manual-result').innerHTML =
                    "<div class='text-blue-400 animate-pulse'>Running aggregation...</div>";

                const params = new URLSearchParams();
                params.append('market_id', marketId);
                params.append('asset_id', assetId);
                params.append('interval', interval);
                params.append('duration', duration);

                fetch('/ui/admin/aggregator/run', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/x-www-form-urlencoded' }},
                    body: params.toString()
                }})
                .then(response => response.text())
                .then(html => {{
                    document.getElementById('manual-result').innerHTML = html;
                    btn.disabled = false;
                    btn.innerHTML = 'Run Aggregation';
                }})
                .catch(error => {{
                    document.getElementById('manual-result').innerHTML =
                        "<div class='text-red-400'>Request failed: " + error + "</div>";
                    btn.disabled = false;
                    btn.innerHTML = 'Run Aggregation';
                }});
            }}

            function runBatchAggregation() {{
                const marketId = document.getElementById('batch-market').value;
                if (!marketId) {{
                    document.getElementById('batch-result').innerHTML =
                        "<div class='text-red-400'>Please select a market</div>";
                    return;
                }}

                const btn = document.getElementById('batch-run-btn');
                const btnText = document.getElementById('batch-btn-text');
                const btnLoading = document.getElementById('batch-btn-loading');

                btn.disabled = true;
                btnText.classList.add('hidden');
                btnLoading.classList.remove('hidden');

                document.getElementById('batch-result').innerHTML =
                    "<div class='text-blue-400 animate-pulse'>Running batch aggregation for all intervals and assets...<br><span class='text-xs text-gray-400'>This may take several minutes</span></div>";

                const params = new URLSearchParams();
                params.append('market_id', marketId);

                fetch('/ui/admin/aggregator/run_batch', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/x-www-form-urlencoded' }},
                    body: params.toString()
                }})
                .then(response => response.text())
                .then(html => {{
                    document.getElementById('batch-result').innerHTML = html;
                    btn.disabled = false;
                    btnText.classList.remove('hidden');
                    btnLoading.classList.add('hidden');
                }})
                .catch(error => {{
                    document.getElementById('batch-result').innerHTML =
                        "<div class='text-red-400'>Request failed: " + error + "</div>";
                    btn.disabled = false;
                    btnText.classList.remove('hidden');
                    btnLoading.classList.add('hidden');
                }});
            }}
        </script>
        "##,
        market_opts = market_opts
    )
}

pub fn admin_accounts_tab(
    assets: Vec<AssetBookRecord>,
    wallets: Vec<CradleWalletAccountRecord>,
) -> String {
    // Build wallet options
    let mut wallet_opts = String::from(r#"<option value="">-- Select from Database --</option>"#);
    for w in &wallets {
        let short_addr = if w.address.len() > 14 {
            format!("{}...{}", &w.address[0..8], &w.address[w.address.len()-6..])
        } else {
            w.address.clone()
        };
        wallet_opts.push_str(&format!(
            r#"<option value="{}">{} ({})</option>"#,
            w.id, short_addr, w.id.to_string().split('-').next().unwrap_or("")
        ));
    }

    // Build asset options
    let mut asset_opts = String::from(r#"<option value="">-- Select Token --</option>"#);
    for a in &assets {
        asset_opts.push_str(&format!(
            r#"<option value="{}">{} ({})</option>"#,
            a.id, a.symbol, a.name
        ));
    }

    format!(
        r##"
        <div class="space-y-6">
            <div class="text-center">
                <h2 class="text-3xl font-bold text-white mb-2">Account Management</h2>
                <p class="text-gray-400">Associate tokens and grant KYC to any wallet address</p>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <!-- Main Form -->
                <div class="bg-gray-800 p-6 rounded-2xl border border-gray-700">
                    <h3 class="text-lg font-bold text-white mb-4">Token Association & KYC</h3>

                    <div class="space-y-4">
                        <!-- Wallet Selection Mode -->
                        <div class="flex gap-2 mb-4">
                            <button type="button" onclick="toggleWalletMode('db')" id="wallet-mode-db"
                                    class="flex-1 py-2 px-4 rounded-lg bg-blue-600 text-white font-medium transition-colors">
                                Select from DB
                            </button>
                            <button type="button" onclick="toggleWalletMode('custom')" id="wallet-mode-custom"
                                    class="flex-1 py-2 px-4 rounded-lg bg-gray-700 text-gray-300 font-medium transition-colors hover:bg-gray-600">
                                Custom Address
                            </button>
                        </div>

                        <!-- DB Wallet Select -->
                        <div id="wallet-db-section">
                            <label class="block text-sm text-gray-400 mb-1">Select Wallet</label>
                            <select id="wallet-select" name="wallet_id"
                                    class="w-full p-3 rounded-lg bg-gray-700 text-white border border-gray-600 focus:ring-2 focus:ring-blue-500">
                                {wallet_opts}
                            </select>
                        </div>

                        <!-- Custom Address Input -->
                        <div id="wallet-custom-section" class="hidden">
                            <label class="block text-sm text-gray-400 mb-1">EVM Address (0x...)</label>
                            <input type="text" id="custom-address" name="custom_address"
                                   placeholder="0x..."
                                   class="w-full p-3 rounded-lg bg-gray-700 text-white border border-gray-600 focus:ring-2 focus:ring-blue-500 font-mono" />
                        </div>

                        <!-- Token Selection -->
                        <div>
                            <label class="block text-sm text-gray-400 mb-1">Select Token</label>
                            <select id="token-select" name="token_id"
                                    class="w-full p-3 rounded-lg bg-gray-700 text-white border border-gray-600 focus:ring-2 focus:ring-blue-500">
                                {asset_opts}
                            </select>
                        </div>

                        <!-- Action Buttons -->
                        <div class="grid grid-cols-3 gap-3 pt-4">
                            <button type="button" onclick="submitAction('associate')"
                                    class="bg-green-600 hover:bg-green-500 text-white font-bold py-3 rounded-lg transition-colors">
                                Associate Only
                            </button>
                            <button type="button" onclick="submitAction('kyc')"
                                    class="bg-yellow-600 hover:bg-yellow-500 text-white font-bold py-3 rounded-lg transition-colors">
                                KYC Only
                            </button>
                            <button type="button" onclick="submitAction('both')"
                                    class="bg-purple-600 hover:bg-purple-500 text-white font-bold py-3 rounded-lg transition-colors">
                                Both
                            </button>
                        </div>

                        <!-- Result Area -->
                        <div id="account-result" class="mt-4"></div>
                    </div>
                </div>

                <!-- Info Panel -->
                <div class="space-y-4">
                    <div class="bg-gray-800/50 p-6 rounded-2xl border border-gray-700">
                        <h4 class="font-bold text-gray-200 mb-3">What is Token Association?</h4>
                        <p class="text-sm text-gray-400">
                            Token association registers a token with a wallet on the Hedera network.
                            A wallet must associate with a token before it can receive or hold that token.
                        </p>
                    </div>

                    <div class="bg-gray-800/50 p-6 rounded-2xl border border-gray-700">
                        <h4 class="font-bold text-gray-200 mb-3">What is KYC?</h4>
                        <p class="text-sm text-gray-400">
                            KYC (Know Your Customer) approval allows a wallet to transact with a token
                            that has KYC requirements enabled. The token's asset manager must grant KYC
                            to each wallet that wants to use the token.
                        </p>
                    </div>

                    <div class="bg-blue-900/30 p-6 rounded-2xl border border-blue-700">
                        <h4 class="font-bold text-blue-300 mb-3">Quick Tips</h4>
                        <ul class="text-sm text-blue-200 space-y-2">
                            <li>• Use "Select from DB" for wallets already registered in the system</li>
                            <li>• Use "Custom Address" for any external EVM address</li>
                            <li>• "Both" will associate then grant KYC in sequence</li>
                            <li>• Some tokens may not require KYC (native tokens)</li>
                        </ul>
                    </div>
                </div>
            </div>
        </div>

        <script>
            let walletMode = 'db';

            function toggleWalletMode(mode) {{
                walletMode = mode;
                const dbBtn = document.getElementById('wallet-mode-db');
                const customBtn = document.getElementById('wallet-mode-custom');
                const dbSection = document.getElementById('wallet-db-section');
                const customSection = document.getElementById('wallet-custom-section');

                if (mode === 'db') {{
                    dbBtn.className = 'flex-1 py-2 px-4 rounded-lg bg-blue-600 text-white font-medium transition-colors';
                    customBtn.className = 'flex-1 py-2 px-4 rounded-lg bg-gray-700 text-gray-300 font-medium transition-colors hover:bg-gray-600';
                    dbSection.classList.remove('hidden');
                    customSection.classList.add('hidden');
                }} else {{
                    customBtn.className = 'flex-1 py-2 px-4 rounded-lg bg-blue-600 text-white font-medium transition-colors';
                    dbBtn.className = 'flex-1 py-2 px-4 rounded-lg bg-gray-700 text-gray-300 font-medium transition-colors hover:bg-gray-600';
                    customSection.classList.remove('hidden');
                    dbSection.classList.add('hidden');
                }}
            }}

            function submitAction(action) {{
                const tokenId = document.getElementById('token-select').value;
                if (!tokenId) {{
                    document.getElementById('account-result').innerHTML =
                        "<div class='text-red-400'>Please select a token</div>";
                    return;
                }}

                let params = new URLSearchParams();
                params.append('token_id', tokenId);

                if (walletMode === 'db') {{
                    const walletId = document.getElementById('wallet-select').value;
                    if (!walletId) {{
                        document.getElementById('account-result').innerHTML =
                            "<div class='text-red-400'>Please select a wallet</div>";
                        return;
                    }}
                    params.append('wallet_id', walletId);
                }} else {{
                    const customAddr = document.getElementById('custom-address').value;
                    if (!customAddr || !customAddr.startsWith('0x')) {{
                        document.getElementById('account-result').innerHTML =
                            "<div class='text-red-400'>Please enter a valid EVM address (0x...)</div>";
                        return;
                    }}
                    params.append('custom_address', customAddr);
                }}

                let endpoint = '/ui/admin/accounts/';
                if (action === 'associate') endpoint += 'associate';
                else if (action === 'kyc') endpoint += 'kyc';
                else endpoint += 'associate_and_kyc';

                document.getElementById('account-result').innerHTML =
                    "<div class='text-blue-400 animate-pulse'>Processing...</div>";

                fetch(endpoint, {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/x-www-form-urlencoded'
                    }},
                    body: params.toString()
                }})
                .then(response => response.text())
                .then(html => {{
                    document.getElementById('account-result').innerHTML = html;
                }})
                .catch(error => {{
                    document.getElementById('account-result').innerHTML =
                        "<div class='text-red-400'>Request failed: " + error + "</div>";
                }});
            }}
        </script>
        "##,
        wallet_opts = wallet_opts,
        asset_opts = asset_opts
    )
}
