
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
    'Mint',
    'RedeemIcpChainKeyAssets'
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
    amount character varying NOT NULL,
    sender character varying,
    receiver character varying NOT NULL,
    memo character varying,
    status public.ticket_status NOT NULL,
    tx_hash character varying
);

CREATE TABLE public.pending_ticket (
    ticket_index integer NOT NULL
);

CREATE SEQUENCE public.pending_ticket_ticket_index_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;
    
ALTER SEQUENCE public.pending_ticket_ticket_index_seq OWNED BY public.pending_ticket.ticket_index;
ALTER TABLE ONLY public.pending_ticket ALTER COLUMN ticket_index SET DEFAULT nextval('public.pending_ticket_ticket_index_seq'::regclass);
COPY public.pending_ticket (ticket_index) FROM stdin;
\.
SELECT pg_catalog.setval('public.pending_ticket_ticket_index_seq', 1, false);

CREATE TABLE public.seaql_migrations (
    version character varying NOT NULL,
    applied_at bigint NOT NULL
);

COPY public.seaql_migrations (version, applied_at) FROM stdin;
m20240507_055143_one	1728346779
m20240701_000001_two	1728346779
m20240802_000001_three	1728346779
m20250111_000001_four	1736654929
\.

CREATE TABLE public.ticket (
    ticket_id text NOT NULL,
    ticket_seq bigint,
    ticket_type public.ticket_type NOT NULL,
    ticket_time bigint NOT NULL,
    src_chain character varying NOT NULL,
    dst_chain character varying NOT NULL,
    action public.tx_action NOT NULL,
    token character varying NOT NULL,
    amount character varying NOT NULL,
    sender character varying,
    receiver character varying NOT NULL,
    memo character varying,
    status public.ticket_status NOT NULL,
    tx_hash character varying,
    intermediate_tx_hash character varying,
    bridge_fee character varying
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

CREATE TABLE public.token_volume (
    token_id character varying NOT NULL,
    ticket_count character varying NOT NULL,
    historical_volume character varying NOT NULL
);

CREATE TABLE public.bridge_fee_log (
    chain_id character varying NOT NULL,
    date character varying NOT NULL,
    fee_token_id character varying NOT NULL,
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

ALTER TABLE ONLY public.token_volume
    ADD CONSTRAINT token_volume_pkey PRIMARY KEY (token_id);

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

ALTER TABLE ONLY public.pending_ticket
    ADD CONSTRAINT pending_ticket_pkey PRIMARY KEY (ticket_index);

ALTER TABLE ONLY public.token_volume
    ADD CONSTRAINT fk_token_id_volume FOREIGN KEY (token_id) REFERENCES public.token_meta(token_id);

ALTER TABLE ONLY public.bridge_fee_log
    ADD CONSTRAINT pk_bridge_fee_log PRIMARY KEY (chain_id, date);

ALTER TABLE ONLY public.bridge_fee_log
    ADD CONSTRAINT fk_log_chain_id FOREIGN KEY (chain_id) REFERENCES public.chain_meta(chain_id);