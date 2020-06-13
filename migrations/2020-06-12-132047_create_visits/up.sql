CREATE TABLE public.visits(
    id bigserial NOT NULL,
    chapter_id integer NOT NULL,
    "timestamp" bigint NOT NULL,
    PRIMARY KEY (id),
    CONSTRAINT chapter_id_fkey FOREIGN KEY (chapter_id)
        REFERENCES public.chapters (id) MATCH SIMPLE
        ON UPDATE NO ACTION
        ON DELETE NO ACTION
);
