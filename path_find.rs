trait path_find {
    // Should always be the same state. Needs knowledge of region ptrs to fix.
    fn get_paths(s: state::state) ->
        (fn @() -> option<(state::state, path::path)>);
}
