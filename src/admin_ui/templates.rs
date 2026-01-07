use cradle_back_end::accounts::db_types::{CradleAccountRecord, CradleWalletAccountRecord};
use cradle_back_end::market::db_types::{MarketRecord, MarketType};
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
            <h1 class="text-3xl font-bold mb-4">Cradle Admin</h1>
            <p>Select an account from the sidebar to begin.</p>
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
        account_id, account_id, account_id, account_id, account_id, account_id
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
        orders_html = r#"<tr><td colspan="5" class="p-4 text-center text-gray-500 italic">No recent orders</td></tr>"#.to_string();
    }

    format!(
         r##"
         <div class="grid grid-cols-1 lg:grid-cols-3 gap-6 h-full">
            <!-- Market Info & Place Order -->
            <div class="lg:col-span-1 space-y-6">
                 <!-- Info Card -->
                 <div class="bg-gray-800 p-6 rounded-xl border border-gray-700">
                     <h3 class="text-xl font-bold text-white mb-2">{}</h3>
                     <p class="text-sm text-gray-400 mb-4">{}</p>
                     <div class="grid grid-cols-2 gap-4 text-sm">
                        <div class="bg-gray-700/50 p-2 rounded">
                            <span class="block text-xs text-gray-500">Asset One</span>
                            <span class="font-mono text-gray-200">{}</span>
                        </div>
                        <div class="bg-gray-700/50 p-2 rounded">
                             <span class="block text-xs text-gray-500">Asset Two</span>
                            <span class="font-mono text-gray-200">{}</span>
                        </div>
                     </div>
                 </div>

                 <!-- Order Form -->
                 <div class="bg-gray-800 p-6 rounded-xl border border-gray-700">
                    <h4 class="text-lg font-bold text-gray-200 mb-4 border-b border-gray-600 pb-2">Place Order</h4>
                    <form hx-post="/ui/order" hx-target="#order-message" hx-on::after-request="this.reset()">
                        <input type="hidden" name="account_id" value="{}" />
                        <input type="hidden" name="market_id" value="{}" />
                        
                        <div class="space-y-4">
                            <div>
                                <label class="block text-xs font-medium text-gray-400 mb-1">Side</label>
                                <div class="grid grid-cols-2 gap-2">
                                    <label class="cursor-pointer">
                                        <input type="radio" name="side" value="buy" class="peer sr-only" checked>
                                        <div class="text-center py-2 rounded bg-gray-700 peer-checked:bg-green-600 peer-checked:text-white transition-all text-sm font-semibold">Buy</div>
                                    </label>
                                    <label class="cursor-pointer">
                                        <input type="radio" name="side" value="sell" class="peer sr-only">
                                        <div class="text-center py-2 rounded bg-gray-700 peer-checked:bg-red-600 peer-checked:text-white transition-all text-sm font-semibold">Sell</div>
                                    </label>
                                </div>
                            </div>
                            
                            <div>
                                <label class="block text-xs font-medium text-gray-400 mb-1">Type</label>
                                <select name="order_type" class="w-full bg-gray-700 border-none rounded p-2 text-sm focus:ring-1 focus:ring-blue-500">
                                    <option value="limit">Limit Order</option>
                                    <option value="market">Market Order</option>
                                </select>
                            </div>

                            <div>
                                <label class="block text-xs font-medium text-gray-400 mb-1">Price</label>
                                <input type="text" name="price" placeholder="0.00" class="w-full bg-gray-700 border-none rounded p-2 text-sm font-mono focus:ring-1 focus:ring-blue-500">
                            </div>

                            <div>
                                <label class="block text-xs font-medium text-gray-400 mb-1">Amount</label>
                                <input type="text" name="amount" placeholder="0.00" class="w-full bg-gray-700 border-none rounded p-2 text-sm font-mono focus:ring-1 focus:ring-blue-500" required>
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
                            hx-get="/ui/market_detail?market_id={}&account_id={}"
                            hx-target="#market-view">Refresh</button>
                </div>
                <div class="overflow-x-auto flex-1">
                    <table class="w-full text-left">
                        <thead class="bg-gray-700/50 text-xs text-gray-400 uppercase">
                            <tr>
                                <th class="px-4 py-2">Side</th>
                                <th class="px-4 py-2">Type</th>
                                <th class="px-4 py-2">Price</th>
                                <th class="px-4 py-2">Bid Amt</th>
                                <th class="px-4 py-2">Ask Amt</th>
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
         "##,
         market.name,
         market.description.unwrap_or_default(),
         market.asset_one,
         market.asset_two,
         account_id,
         market.id,
         market.id, account_id
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
