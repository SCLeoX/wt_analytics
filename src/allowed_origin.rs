pub fn is_allowed_origin(origin: &str) -> bool {
    matches! { origin,
          "https://wt.tepis.me"
        | "https://wt.bgme.me"
        | "https://rbq.desi"
        | "https://wt.makai.city"
        | "https://wt.0w0.bid"
        // | "http://localhost:2333"
        // | "http://127.0.0.1:2333"
    }
}
