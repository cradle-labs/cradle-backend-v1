-- This file should undo anything in `up.sql`

alter table Loans drop column if exists transaction;

alter table LoanRepayments drop column if exists transaction;

alter table LoanLiquidations drop column if exists transaction;