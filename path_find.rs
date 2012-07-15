trait path_find {
    // Should always be the same state. Needs knowledge of region ptrs to fix.
    fn next_target_path(s: state::state)
        -> option<(state::state, path::path)>;
}
