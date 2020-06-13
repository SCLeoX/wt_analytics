CREATE TABLE public.chapters(
    id serial NOT NULL,
    relative_path character varying(1024) NOT NULL,
    visit_count bigint NOT NULL DEFAULT 0,
    PRIMARY KEY (id),
    CONSTRAINT relative_path_unique UNIQUE (relative_path)
);
