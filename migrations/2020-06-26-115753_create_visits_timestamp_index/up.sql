CREATE INDEX visits_timestamp_index
    ON public.visits USING btree
        ("timestamp" ASC NULLS LAST);