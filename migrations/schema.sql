--
-- PostgreSQL database dump
--

\restrict PW17xzJyqf9QqnB9Mwflpk6jLUCafC8XjLpfGw6VcXODMFevuuGxYrMJtCH1lhk

-- Dumped from database version 14.20 (Ubuntu 14.20-0ubuntu0.22.04.1)
-- Dumped by pg_dump version 14.20 (Ubuntu 14.20-0ubuntu0.22.04.1)

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


ALTER TABLE public.archive_tags_tag_id_seq OWNER TO postgres;

--
-- Name: archive_tags_tag_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.archive_tags_tag_id_seq OWNED BY public.archive_tags.tag_id;


--
-- Name: users; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.users (
    id integer NOT NULL,
    username character varying(32) NOT NULL,
    phash character varying NOT NULL,
    psalt bytea NOT NULL,
    must_change_password boolean
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


ALTER TABLE public.users_id_seq OWNER TO postgres;

--
-- Name: users_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.users_id_seq OWNED BY public.users.id;


--
-- Name: archive_tags tag_id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.archive_tags ALTER COLUMN tag_id SET DEFAULT nextval('public.archive_tags_tag_id_seq'::regclass);


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
-- PostgreSQL database dump complete
--

\unrestrict PW17xzJyqf9QqnB9Mwflpk6jLUCafC8XjLpfGw6VcXODMFevuuGxYrMJtCH1lhk