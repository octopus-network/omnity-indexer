
CREATE TYPE public.chain_state AS ENUM (
    'Active',
    'Deactive'
);

CREATE TYPE public.chain_type AS ENUM (
    'SettlementChain',
    'ExecutionChain'
);

CREATE TYPE public.ticket_status AS ENUM (
    'Unknown',
    'WaitingForConfirmBySrc',
    'WaitingForConfirmByDest',
    'Finalized',
    'Pending'
);

CREATE TYPE public.ticket_type AS ENUM (
    'Normal',
    'Resubmit'
);

CREATE TYPE public.tx_action AS ENUM (
    'Transfer',
    'Redeem',
    'Burn',
    'Mint'
);

SET default_tablespace = '';

SET default_table_access_method = heap;

CREATE TABLE public.chain_meta (
    chain_id character varying NOT NULL,
    canister_id text NOT NULL,
    chain_type public.chain_type NOT NULL,
    chain_state public.chain_state NOT NULL,
    contract_address character varying,
    counterparties json,
    fee_token character varying
);

CREATE TABLE public.deleted_mint_ticket (
    ticket_id text NOT NULL,
    ticket_seq bigint,
    ticket_type public.ticket_type NOT NULL,
    ticket_time bigint NOT NULL,
    src_chain character varying NOT NULL,
    dst_chain character varying NOT NULL,
    action public.tx_action NOT NULL,
    token character varying NOT NULL,
    amount bigint NOT NULL,
    sender character varying,
    receiver character varying NOT NULL,
    memo bytea,
    status public.ticket_status NOT NULL,
    tx_hash character varying
);

CREATE TABLE public.seaql_migrations (
    version character varying NOT NULL,
    applied_at bigint NOT NULL
);

CREATE TABLE public.ticket (
    ticket_id text NOT NULL,
    ticket_seq bigint,
    ticket_type public.ticket_type NOT NULL,
    ticket_time bigint NOT NULL,
    src_chain character varying NOT NULL,
    dst_chain character varying NOT NULL,
    action public.tx_action NOT NULL,
    token character varying NOT NULL,
    amount bigint NOT NULL,
    sender character varying,
    receiver character varying NOT NULL,
    memo bytea,
    status public.ticket_status NOT NULL,
    tx_hash character varying,
    intermediate_tx_hash character varying
);

CREATE TABLE public.token_ledger_id_on_chain (
    chain_id character varying NOT NULL,
    token_id character varying NOT NULL,
    contract_id character varying NOT NULL
);

CREATE TABLE public.token_meta (
    token_id character varying NOT NULL,
    name character varying NOT NULL,
    symbol character varying NOT NULL,
    issue_chain character varying NOT NULL,
    decimals smallint NOT NULL,
    icon text,
    metadata json NOT NULL,
    dst_chains json NOT NULL
);

CREATE TABLE public.token_on_chain (
    chain_id character varying NOT NULL,
    token_id character varying NOT NULL,
    amount character varying NOT NULL
);

ALTER TABLE ONLY public.chain_meta
    ADD CONSTRAINT chain_meta_pkey PRIMARY KEY (chain_id);

ALTER TABLE ONLY public.deleted_mint_ticket
    ADD CONSTRAINT deleted_mint_ticket_pkey PRIMARY KEY (ticket_id);

ALTER TABLE ONLY public.token_on_chain
    ADD CONSTRAINT pk_chain_token PRIMARY KEY (chain_id, token_id);


ALTER TABLE ONLY public.token_ledger_id_on_chain
    ADD CONSTRAINT pk_chain_token_contract PRIMARY KEY (chain_id, token_id);


ALTER TABLE ONLY public.seaql_migrations
    ADD CONSTRAINT seaql_migrations_pkey PRIMARY KEY (version);


ALTER TABLE ONLY public.ticket
    ADD CONSTRAINT ticket_pkey PRIMARY KEY (ticket_id);


ALTER TABLE ONLY public.token_meta
    ADD CONSTRAINT token_meta_pkey PRIMARY KEY (token_id);

CREATE INDEX "idx-mint-ticket_seq" ON public.deleted_mint_ticket USING btree (ticket_seq);

CREATE INDEX "idx-ticket_seq" ON public.ticket USING btree (ticket_seq);

ALTER TABLE ONLY public.token_on_chain
    ADD CONSTRAINT fk_chain_id FOREIGN KEY (chain_id) REFERENCES public.chain_meta(chain_id);


ALTER TABLE ONLY public.token_ledger_id_on_chain
    ADD CONSTRAINT fk_chain_id FOREIGN KEY (chain_id) REFERENCES public.chain_meta(chain_id);


ALTER TABLE ONLY public.token_on_chain
    ADD CONSTRAINT fk_token_id FOREIGN KEY (token_id) REFERENCES public.token_meta(token_id);


ALTER TABLE ONLY public.token_ledger_id_on_chain
    ADD CONSTRAINT fk_token_id FOREIGN KEY (token_id) REFERENCES public.token_meta(token_id);
