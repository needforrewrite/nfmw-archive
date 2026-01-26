-- Add up migration script here

--
-- PostgreSQL database dump
--

-- Dumped from database version 18.1
-- Dumped by pg_dump version 18.1

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: archive_item_tags; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.archive_item_tags (
    archive_item_id uuid NOT NULL,
    tag_id integer NOT NULL
);


ALTER TABLE public.archive_item_tags OWNER TO postgres;

--
-- Name: archive_items; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.archive_items (
    archive_item_id uuid NOT NULL,
    path text NOT NULL,
    type text NOT NULL,
    name text NOT NULL,
    created_at timestamp without time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    author text,
    owner_user_id integer
);


ALTER TABLE public.archive_items OWNER TO postgres;

--
-- Name: archive_tags; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.archive_tags (
    tag_id integer NOT NULL,
    tag_name text NOT NULL
);


ALTER TABLE public.archive_tags OWNER TO postgres;

--
-- Name: archive_tags_ownership; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.archive_tags_ownership (
    tag_id integer,
    user_id integer
);


ALTER TABLE public.archive_tags_ownership OWNER TO postgres;

--
-- Name: archive_tags_tag_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.archive_tags_tag_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.archive_tags_tag_id_seq OWNER TO postgres;

--
-- Name: archive_tags_tag_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.archive_tags_tag_id_seq OWNED BY public.archive_tags.tag_id;


--
-- Name: discord_oauth2; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.discord_oauth2 (
    entry_id integer NOT NULL,
    user_id integer NOT NULL,
    discord_user_id bigint NOT NULL,
    created_at timestamp without time zone DEFAULT now()
);


ALTER TABLE public.discord_oauth2 OWNER TO postgres;

--
-- Name: discord_oauth2_entry_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.discord_oauth2_entry_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.discord_oauth2_entry_id_seq OWNER TO postgres;

--
-- Name: discord_oauth2_entry_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.discord_oauth2_entry_id_seq OWNED BY public.discord_oauth2.entry_id;


--
-- Name: discord_oauth2_token_storage; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.discord_oauth2_token_storage (
    session_token text NOT NULL,
    discord_token text NOT NULL,
    created_at timestamp without time zone DEFAULT now(),
    discord_user_id bigint NOT NULL
);


ALTER TABLE public.discord_oauth2_token_storage OWNER TO postgres;

--
-- Name: user_tokens; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.user_tokens (
    token_id integer NOT NULL,
    user_id integer NOT NULL,
    token text NOT NULL
);


ALTER TABLE public.user_tokens OWNER TO postgres;

--
-- Name: user_tokens_token_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.user_tokens_token_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.user_tokens_token_id_seq OWNER TO postgres;

--
-- Name: user_tokens_token_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.user_tokens_token_id_seq OWNED BY public.user_tokens.token_id;


--
-- Name: users; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.users (
    id integer NOT NULL,
    username character varying(32) NOT NULL,
    phash character varying,
    psalt bytea,
    must_change_password boolean,
    created_at timestamp without time zone DEFAULT now()
);


ALTER TABLE public.users OWNER TO postgres;

--
-- Name: users_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.users_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.users_id_seq OWNER TO postgres;

--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: archive_tags tag_id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.archive_tags ALTER COLUMN tag_id SET DEFAULT nextval('public.archive_tags_tag_id_seq'::regclass);


--
-- Name: discord_oauth2 entry_id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.discord_oauth2 ALTER COLUMN entry_id SET DEFAULT nextval('public.discord_oauth2_entry_id_seq'::regclass);


--
-- Name: user_tokens token_id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.user_tokens ALTER COLUMN token_id SET DEFAULT nextval('public.user_tokens_token_id_seq'::regclass);


--
-- Name: users id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.users ALTER COLUMN id SET DEFAULT nextval('public.users_id_seq'::regclass);


--
-- Name: archive_item_tags archive_item_tags_archive_item_id_tag_id_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.archive_item_tags
    ADD CONSTRAINT archive_item_tags_archive_item_id_tag_id_key UNIQUE (archive_item_id, tag_id);


--
-- Name: archive_items archive_items_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.archive_items
    ADD CONSTRAINT archive_items_pkey PRIMARY KEY (archive_item_id);


--
-- Name: archive_tags_ownership archive_tags_ownership_tag_id_user_id_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.archive_tags_ownership
    ADD CONSTRAINT archive_tags_ownership_tag_id_user_id_key UNIQUE (tag_id, user_id);


--
-- Name: archive_tags archive_tags_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.archive_tags
    ADD CONSTRAINT archive_tags_pkey PRIMARY KEY (tag_id);


--
-- Name: archive_tags archive_tags_tag_name_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.archive_tags
    ADD CONSTRAINT archive_tags_tag_name_key UNIQUE (tag_name);


--
-- Name: discord_oauth2 discord_oauth2_discord_user_id_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.discord_oauth2
    ADD CONSTRAINT discord_oauth2_discord_user_id_key UNIQUE (discord_user_id);


--
-- Name: discord_oauth2 discord_oauth2_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.discord_oauth2
    ADD CONSTRAINT discord_oauth2_pkey PRIMARY KEY (entry_id);


--
-- Name: discord_oauth2_token_storage discord_oauth2_token_storage_discord_user_id_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.discord_oauth2_token_storage
    ADD CONSTRAINT discord_oauth2_token_storage_discord_user_id_key UNIQUE (discord_user_id);


--
-- Name: discord_oauth2_token_storage discord_oauth2_token_storage_session_token_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.discord_oauth2_token_storage
    ADD CONSTRAINT discord_oauth2_token_storage_session_token_key UNIQUE (session_token);


--
-- Name: discord_oauth2 discord_oauth2_user_id_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.discord_oauth2
    ADD CONSTRAINT discord_oauth2_user_id_key UNIQUE (user_id);


--
-- Name: user_tokens user_tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.user_tokens
    ADD CONSTRAINT user_tokens_pkey PRIMARY KEY (token_id);


--
-- Name: user_tokens user_tokens_user_id_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.user_tokens
    ADD CONSTRAINT user_tokens_user_id_key UNIQUE (user_id);


--
-- Name: users users_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_pkey PRIMARY KEY (id);


--
-- Name: users users_username_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.users
    ADD CONSTRAINT users_username_key UNIQUE (username);


--
-- Name: archive_item_tags archive_item_tags_archive_item_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.archive_item_tags
    ADD CONSTRAINT archive_item_tags_archive_item_id_fkey FOREIGN KEY (archive_item_id) REFERENCES public.archive_items(archive_item_id) ON DELETE CASCADE;


--
-- Name: archive_item_tags archive_item_tags_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.archive_item_tags
    ADD CONSTRAINT archive_item_tags_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.archive_tags(tag_id) ON DELETE CASCADE;


--
-- Name: archive_items archive_items_owner_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.archive_items
    ADD CONSTRAINT archive_items_owner_user_id_fkey FOREIGN KEY (owner_user_id) REFERENCES public.users(id) ON DELETE SET NULL;


--
-- Name: archive_tags_ownership archive_tags_ownership_tag_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.archive_tags_ownership
    ADD CONSTRAINT archive_tags_ownership_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.archive_tags(tag_id) ON DELETE CASCADE;


--
-- Name: archive_tags_ownership archive_tags_ownership_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.archive_tags_ownership
    ADD CONSTRAINT archive_tags_ownership_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: discord_oauth2 discord_oauth2_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.discord_oauth2
    ADD CONSTRAINT discord_oauth2_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- Name: user_tokens user_tokens_user_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.user_tokens
    ADD CONSTRAINT user_tokens_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;


--
-- PostgreSQL database dump complete
--

