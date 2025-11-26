-- This file should undo anything in `up.sql`

alter type CradleAccountType rename to OldCradleAccountType;

create type NCradleAccountType as enum ('retail', 'institutional');

alter table CradleAccounts alter column account_type type NCradleAccountType using account_type::text::NCradleAccountType;

drop type OldCradleAccountType;

alter type NCradleAccountType rename to CradleAccountType;

-- TODO: at the moment this fails but will need to update it later so that spinning down stuff doesnt get hard
