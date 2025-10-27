-- Your SQL goes here

create type loan_status as enum ('active', 'repaid', 'liquidated');

create table if not exists Loans (
    id uuid primary key not null default uuid_generate_v4(),
    account_id uuid not null references CradleAccounts(id),
    wallet_id uuid not null references CradleWalletAccounts(id),
    pool uuid not null references LendingPool(id),
    borrow_index numeric not null,
    principal_amount numeric not null,
    created_at timestamp not null default now(),
    status loan_status not null default 'active'
);

create table if not exists LoanRepayments (
    id uuid primary key not null default uuid_generate_v4(),
    loan_id uuid not null references Loans(id),
    repayment_amount numeric not null,
    repayment_date timestamp not null default now()
);

create table if not exists LoanLiquidations (
    id uuid primary key not null default uuid_generate_v4(),
    loan_id uuid not null references Loans(id),
    liquidator_wallet_id uuid not null references CradleWalletAccounts(id),
    liquidation_amount numeric not null,
    liquidation_date timestamp not null default now()
);

create index if not exists idx_loans_account_id on Loans(account_id);
create index if not exists idx_loans_wallet_id on Loans(wallet_id);
create index if not exists idx_loans_pool on Loans(pool);
create index if not exists idx_loanrepayments_loan_id on LoanRepayments(loan_id);
create index if not exists idx_loanliquidations_loan_id on LoanLiquidations(loan_id);
create index if not exists idx_loanliquidations_liquidator_wallet_id on LoanLiquidations(liquidator_wallet_id);