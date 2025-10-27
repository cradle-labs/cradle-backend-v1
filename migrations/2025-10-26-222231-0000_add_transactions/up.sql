-- Your SQL goes here
alter table Loans add column transaction text default '';

alter table LoanRepayments add column transaction text default '';

alter table LoanLiquidations add column transaction text default '';