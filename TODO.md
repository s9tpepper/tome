        #[cfg(not(debug_assertions))]
        let set = SyntaxSet::load_defaults_newlines();
        #[cfg(debug_assertions)]
        let set = SyntaxSet::new();


// Create these only once
pub struct Highlighter {
    set: SyntaxSet,
    theme_set: ThemeSet,
}
