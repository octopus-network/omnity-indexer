--
-- PostgreSQL database dump
--

-- Dumped from database version 12.19 (Debian 12.19-1.pgdg120+1)
-- Dumped by pg_dump version 16.3 (Homebrew)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: public; Type: SCHEMA; Schema: -; Owner: postgres
--

-- *not* creating schema, since initdb creates it


ALTER SCHEMA public OWNER TO postgres;

--
-- Name: chain_state; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public.chain_state AS ENUM (
    'Active',
    'Deactive'
);


ALTER TYPE public.chain_state OWNER TO postgres;

--
-- Name: chain_type; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public.chain_type AS ENUM (
    'SettlementChain',
    'ExecutionChain'
);


ALTER TYPE public.chain_type OWNER TO postgres;

--
-- Name: ticket_status; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public.ticket_status AS ENUM (
    'Unknown',
    'WaitingForConfirmBySrc',
    'WaitingForConfirmByDest',
    'Finalized'
);


ALTER TYPE public.ticket_status OWNER TO postgres;

--
-- Name: ticket_type; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public.ticket_type AS ENUM (
    'Normal',
    'Resubmit'
);


ALTER TYPE public.ticket_type OWNER TO postgres;

--
-- Name: tx_action; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public.tx_action AS ENUM (
    'Transfer',
    'Redeem',
    'Burn',
    'Mint'
);


ALTER TYPE public.tx_action OWNER TO postgres;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: chain_meta; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.chain_meta (
    chain_id character varying NOT NULL,
    canister_id text NOT NULL,
    chain_type public.chain_type NOT NULL,
    chain_state public.chain_state NOT NULL,
    contract_address character varying,
    counterparties json,
    fee_token character varying
);


ALTER TABLE public.chain_meta OWNER TO postgres;

--
-- Name: seaql_migrations; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.seaql_migrations (
    version character varying NOT NULL,
    applied_at bigint NOT NULL
);


ALTER TABLE public.seaql_migrations OWNER TO postgres;

--
-- Name: ticket; Type: TABLE; Schema: public; Owner: postgres
--

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
    memo bytea,
    status public.ticket_status NOT NULL
);


ALTER TABLE public.ticket OWNER TO postgres;

--
-- Name: token_meta; Type: TABLE; Schema: public; Owner: postgres
--

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


ALTER TABLE public.token_meta OWNER TO postgres;

--
-- Name: token_on_chain; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.token_on_chain (
    chain_id character varying NOT NULL,
    token_id character varying NOT NULL,
    amount character varying NOT NULL
);


ALTER TABLE public.token_on_chain OWNER TO postgres;

--
-- Data for Name: chain_meta; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.chain_meta (chain_id, canister_id, chain_type, chain_state, contract_address, counterparties, fee_token) FROM stdin;
\.


--
-- Data for Name: seaql_migrations; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.seaql_migrations (version, applied_at) FROM stdin;
m20240507_055143_one	1720530904
m20240701_000001_two	1720530904
\.


--
-- Data for Name: ticket; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.ticket (ticket_id, ticket_seq, ticket_type, ticket_time, src_chain, dst_chain, action, token, amount, sender, receiver, memo, status) FROM stdin;
\.


--
-- Data for Name: token_meta; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.token_meta (token_id, name, symbol, issue_chain, decimals, icon, metadata, dst_chains) FROM stdin;
\.


--
-- Data for Name: token_on_chain; Type: TABLE DATA; Schema: public; Owner: postgres
--

COPY public.token_on_chain (chain_id, token_id, amount) FROM stdin;
\.


--
-- Name: chain_meta chain_meta_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.chain_meta
    ADD CONSTRAINT chain_meta_pkey PRIMARY KEY (chain_id);


--
-- Name: token_on_chain pk_chain_token; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.token_on_chain
    ADD CONSTRAINT pk_chain_token PRIMARY KEY (chain_id, token_id);


--
-- Name: seaql_migrations seaql_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.seaql_migrations
    ADD CONSTRAINT seaql_migrations_pkey PRIMARY KEY (version);


--
-- Name: ticket ticket_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.ticket
    ADD CONSTRAINT ticket_pkey PRIMARY KEY (ticket_id);


--
-- Name: token_meta token_meta_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.token_meta
    ADD CONSTRAINT token_meta_pkey PRIMARY KEY (token_id);


--
-- Name: idx-ticket_seq; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX "idx-ticket_seq" ON public.ticket USING btree (ticket_seq);


--
-- Name: token_on_chain fk_chain_id; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.token_on_chain
    ADD CONSTRAINT fk_chain_id FOREIGN KEY (chain_id) REFERENCES public.chain_meta(chain_id);


--
-- Name: token_on_chain fk_token_id; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.token_on_chain
    ADD CONSTRAINT fk_token_id FOREIGN KEY (token_id) REFERENCES public.token_meta(token_id);


--
-- Name: SCHEMA public; Type: ACL; Schema: -; Owner: postgres
--

REVOKE USAGE ON SCHEMA public FROM PUBLIC;
GRANT ALL ON SCHEMA public TO PUBLIC;


--
-- PostgreSQL database dump complete
--

