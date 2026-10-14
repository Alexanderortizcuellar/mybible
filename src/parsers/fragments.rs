use crate::error::{MyBibleError, Result};
use regex::Regex;
use std::collections::HashSet;

/// Parsed metadata for an `<!-- INCLUDE(...) -->` HTML comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentInclude {
    pub fragment_id: String,
    pub from_module: Option<String>,
    pub textual: bool,
    pub matched_tag: String,
}

lazy_static::lazy_static! {
    static ref INCLUDE_REGEX: Regex = Regex::new(
        r"<!--\s*INCLUDE\(([^)]+)\)(?:\s+FROM\(([^)]+)\))?(?:\s+TEXTUAL\((true|false)\))?\s*-->"
    ).unwrap();
}

/// Find all fragment include directives in the given HTML string.
pub fn parse_fragment_includes(html: &str) -> Vec<FragmentInclude> {
    let mut includes = Vec::new();
    for cap in INCLUDE_REGEX.captures_iter(html) {
        let fragment_id = cap.get(1).map_or("", |m| m.as_str()).trim().to_string();
        let from_module = cap.get(2).map(|m| m.as_str().trim().to_string());
        let textual = cap
            .get(3)
            .is_some_and(|m| m.as_str().eq_ignore_ascii_case("true"));
        let matched_tag = cap.get(0).map_or("", |m| m.as_str()).to_string();

        includes.push(FragmentInclude {
            fragment_id,
            from_module,
            textual,
            matched_tag,
        });
    }
    includes
}

/// Trait for resolving content fragments across modules.
pub trait FragmentResolver {
    fn get_fragment(&self, fragment_id: &str, module: Option<&str>) -> Option<String>;
}

/// Expand all `<!-- INCLUDE(...) -->` comments in the HTML recursively with cycle prevention.
pub fn expand_fragments<R: FragmentResolver>(
    html: &str,
    current_module: Option<&str>,
    resolver: &R,
) -> Result<String> {
    let mut visited = HashSet::new();
    expand_internal(html, current_module, resolver, &mut visited, 0)
}

fn expand_internal<R: FragmentResolver>(
    html: &str,
    current_module: Option<&str>,
    resolver: &R,
    visited: &mut HashSet<String>,
    depth: usize,
) -> Result<String> {
    if depth > 32 {
        return Err(MyBibleError::CircularFragmentInclusion(
            "Maximum inclusion depth exceeded (32)".to_string(),
        ));
    }

    let includes = parse_fragment_includes(html);
    if includes.is_empty() {
        return Ok(html.to_string());
    }

    let mut result = html.to_string();
    for inc in includes {
        let target_module = inc.from_module.as_deref().or(current_module);
        let key = format!("{}:{}", target_module.unwrap_or(""), inc.fragment_id);

        if visited.contains(&key) {
            return Err(MyBibleError::CircularFragmentInclusion(inc.fragment_id));
        }

        if let Some(content) = resolver.get_fragment(&inc.fragment_id, target_module) {
            visited.insert(key.clone());
            let expanded_child =
                expand_internal(&content, target_module, resolver, visited, depth + 1)?;
            visited.remove(&key);
            result = result.replace(&inc.matched_tag, &expanded_child);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct MockResolver {
        fragments: HashMap<(String, Option<String>), String>,
    }

    impl FragmentResolver for MockResolver {
        fn get_fragment(&self, fragment_id: &str, module: Option<&str>) -> Option<String> {
            self.fragments
                .get(&(fragment_id.to_string(), module.map(|m| m.to_string())))
                .cloned()
        }
    }

    #[test]
    fn test_expand_fragments() {
        let mut map = HashMap::new();
        map.insert(
            ("logo".to_string(), None),
            "<img src='logo.png'/>".to_string(),
        );
        map.insert(
            ("cross".to_string(), Some("GSD".to_string())),
            "<b>Cross Ref</b>".to_string(),
        );

        let resolver = MockResolver { fragments: map };
        let html = "Start <!-- INCLUDE(logo) --> middle <!-- INCLUDE(cross) FROM(GSD) TEXTUAL(true) --> end";
        let res = expand_fragments(html, None, &resolver).unwrap();

        assert_eq!(
            res,
            "Start <img src='logo.png'/> middle <b>Cross Ref</b> end"
        );
    }
}
