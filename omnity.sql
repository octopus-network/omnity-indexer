--
-- PostgreSQL database dump
--

-- Dumped from database version 12.18 (Debian 12.18-1.pgdg120+2)
-- Dumped by pg_dump version 12.18 (Debian 12.18-1.pgdg120+2)

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
-- Name: chain_state; Type: TYPE; Schema: public; Owner: omnity
--

CREATE TYPE public.chain_state AS ENUM (
    'Active',
    'Deactive'
);


ALTER TYPE public.chain_state OWNER TO omnity;

--
-- Name: chain_type; Type: TYPE; Schema: public; Owner: omnity
--

CREATE TYPE public.chain_type AS ENUM (
    'SettlementChain',
    'ExecutionChain'
);


ALTER TYPE public.chain_type OWNER TO omnity;

--
-- Name: ticket_status; Type: TYPE; Schema: public; Owner: omnity
--

CREATE TYPE public.ticket_status AS ENUM (
    'Unknown',
    'WaitingForConfirmBySrc',
    'WaitingForConfirmByDest',
    'Finalized'
);


ALTER TYPE public.ticket_status OWNER TO omnity;

--
-- Name: ticket_type; Type: TYPE; Schema: public; Owner: omnity
--

CREATE TYPE public.ticket_type AS ENUM (
    'Normal',
    'Resubmit'
);


ALTER TYPE public.ticket_type OWNER TO omnity;

--
-- Name: tx_action; Type: TYPE; Schema: public; Owner: omnity
--

CREATE TYPE public.tx_action AS ENUM (
    'Transfer',
    'Redeem',
    'Burn'
);


ALTER TYPE public.tx_action OWNER TO omnity;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: chain_meta; Type: TABLE; Schema: public; Owner: omnity
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


ALTER TABLE public.chain_meta OWNER TO omnity;

--
-- Name: seaql_migrations; Type: TABLE; Schema: public; Owner: omnity
--

CREATE TABLE public.seaql_migrations (
    version character varying NOT NULL,
    applied_at bigint NOT NULL
);


ALTER TABLE public.seaql_migrations OWNER TO omnity;

--
-- Name: ticket; Type: TABLE; Schema: public; Owner: omnity
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


ALTER TABLE public.ticket OWNER TO omnity;

--
-- Name: token_meta; Type: TABLE; Schema: public; Owner: omnity
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


ALTER TABLE public.token_meta OWNER TO omnity;

--
-- Name: token_on_chain; Type: TABLE; Schema: public; Owner: omnity
--

CREATE TABLE public.token_on_chain (
    chain_id character varying NOT NULL,
    token_id character varying NOT NULL,
    amount character varying NOT NULL
);


ALTER TABLE public.token_on_chain OWNER TO omnity;

--
-- Name: chain_meta chain_meta_pkey; Type: CONSTRAINT; Schema: public; Owner: omnity
--

ALTER TABLE ONLY public.chain_meta
    ADD CONSTRAINT chain_meta_pkey PRIMARY KEY (chain_id);


--
-- Name: token_on_chain pk_chain_token; Type: CONSTRAINT; Schema: public; Owner: omnity
--

ALTER TABLE ONLY public.token_on_chain
    ADD CONSTRAINT pk_chain_token PRIMARY KEY (chain_id, token_id);


--
-- Name: seaql_migrations seaql_migrations_pkey; Type: CONSTRAINT; Schema: public; Owner: omnity
--

ALTER TABLE ONLY public.seaql_migrations
    ADD CONSTRAINT seaql_migrations_pkey PRIMARY KEY (version);


--
-- Name: ticket ticket_pkey; Type: CONSTRAINT; Schema: public; Owner: omnity
--

ALTER TABLE ONLY public.ticket
    ADD CONSTRAINT ticket_pkey PRIMARY KEY (ticket_id);


--
-- Name: token_meta token_meta_pkey; Type: CONSTRAINT; Schema: public; Owner: omnity
--

ALTER TABLE ONLY public.token_meta
    ADD CONSTRAINT token_meta_pkey PRIMARY KEY (token_id);


--
-- Name: idx-ticket_seq; Type: INDEX; Schema: public; Owner: omnity
--

CREATE INDEX "idx-ticket_seq" ON public.ticket USING btree (ticket_seq);

--
-- Name: token_on_chain fk_chain_id; Type: FK CONSTRAINT; Schema: public; Owner: omnity
--

ALTER TABLE ONLY public.token_on_chain
    ADD CONSTRAINT fk_chain_id FOREIGN KEY (chain_id) REFERENCES public.chain_meta(chain_id);


--
-- Name: token_on_chain fk_token_id; Type: FK CONSTRAINT; Schema: public; Owner: omnity
--

ALTER TABLE ONLY public.token_on_chain
    ADD CONSTRAINT fk_token_id FOREIGN KEY (token_id) REFERENCES public.token_meta(token_id);



--
-- PostgreSQL database dump complete
--

