//! Language support and tree-sitter grammars for AST operations.

/// Built-in ast-grep languages wired to their tree-sitter grammars.
#[derive(::core::clone::Clone, ::core::marker::Copy, ::core::fmt::Debug, ::core::cmp::PartialEq, ::core::cmp::Eq)]
pub(crate) enum SupportedLanguage {
    Rust,
    TypeScript,
    Tsx,
    JavaScript,
    Python,
    Json,
    Toml,
    Markdown,
}

impl ::ast_grep_core::Language for self::SupportedLanguage {
    fn kind_to_id(&self, kind: &::core::primitive::str) -> ::core::primitive::u16 {
        let ts_language = self.ts_language();
        ts_language.id_for_node_kind(kind, true)
    }

    fn field_to_id(&self, field: &::core::primitive::str) -> ::core::option::Option<::core::primitive::u16> {
        let ts_language = self.ts_language();
        ts_language.field_id_for_name(field).map(|field_id| field_id.get())
    }

    fn build_pattern(
        &self,
        builder: &::ast_grep_core::matcher::PatternBuilder,
    ) -> ::core::result::Result<::ast_grep_core::Pattern, ::ast_grep_core::PatternError> {
        let language_copy = ::core::clone::Clone::clone(self);
        builder.build(|pattern_src| {
            ::ast_grep_core::tree_sitter::StrDoc::try_new(pattern_src, language_copy)
        })
    }
}

impl ::ast_grep_core::tree_sitter::LanguageExt for self::SupportedLanguage {
    fn get_ts_language(&self) -> ::ast_grep_core::tree_sitter::TSLanguage {
        self.ts_language()
    }
}

impl self::SupportedLanguage {
    fn ts_language(&self) -> ::ast_grep_core::tree_sitter::TSLanguage {
        match self {
            self::SupportedLanguage::Rust => ::tree_sitter_rust::LANGUAGE.into(),
            self::SupportedLanguage::TypeScript => ::tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            self::SupportedLanguage::Tsx => ::tree_sitter_typescript::LANGUAGE_TSX.into(),
            self::SupportedLanguage::JavaScript => ::tree_sitter_javascript::LANGUAGE.into(),
            self::SupportedLanguage::Python => ::tree_sitter_python::LANGUAGE.into(),
            self::SupportedLanguage::Json => ::tree_sitter_json::LANGUAGE.into(),
            self::SupportedLanguage::Toml => ::tree_sitter_toml_ng::LANGUAGE.into(),
            self::SupportedLanguage::Markdown => ::tree_sitter_md::LANGUAGE.into(),
        }
    }
}

/// Infers the ast-grep language from a file extension.
pub(crate) fn infer_file_language(file_name: &::core::primitive::str) -> ::core::option::Option<self::SupportedLanguage> {
    let lower_name = file_name.to_ascii_lowercase();
    if lower_name.ends_with(".rs") {
        return ::core::option::Option::Some(self::SupportedLanguage::Rust);
    }
    if lower_name.ends_with(".ts") {
        return ::core::option::Option::Some(self::SupportedLanguage::TypeScript);
    }
    if lower_name.ends_with(".tsx") {
        return ::core::option::Option::Some(self::SupportedLanguage::Tsx);
    }
    if lower_name.ends_with(".js") || lower_name.ends_with(".jsx") {
        return ::core::option::Option::Some(self::SupportedLanguage::JavaScript);
    }
    if lower_name.ends_with(".py") {
        return ::core::option::Option::Some(self::SupportedLanguage::Python);
    }
    if lower_name.ends_with(".json") {
        return ::core::option::Option::Some(self::SupportedLanguage::Json);
    }
    if lower_name.ends_with(".toml") {
        return ::core::option::Option::Some(self::SupportedLanguage::Toml);
    }
    if lower_name.ends_with(".md") {
        return ::core::option::Option::Some(self::SupportedLanguage::Markdown);
    }
    ::core::option::Option::None
}

/// Resolves a language filter identifier into an ast-grep language instance.
pub(crate) fn resolve_language(lang_name: &::core::primitive::str) -> ::core::result::Result<self::SupportedLanguage, ::std::string::String> {
    let resolved = match lang_name {
        "rust" | "rs" => self::SupportedLanguage::Rust,
        "typescript" | "ts" => self::SupportedLanguage::TypeScript,
        "tsx" => self::SupportedLanguage::Tsx,
        "javascript" | "js" => self::SupportedLanguage::JavaScript,
        "python" | "py" => self::SupportedLanguage::Python,
        "json" => self::SupportedLanguage::Json,
        "toml" => self::SupportedLanguage::Toml,
        "markdown" | "md" => self::SupportedLanguage::Markdown,
        other => return ::core::result::Result::Err(::std::format!("unsupported ast-grep language filter: {other}")),
    };
    ::core::result::Result::Ok(resolved)
}

/// Reports whether a file matches the requested language filter.
pub(crate) fn matches_language(
    file_name: &::core::primitive::str,
    lang_filter: ::core::option::Option<&::core::primitive::str>,
) -> ::core::primitive::bool {
    let filter = match lang_filter {
        ::core::option::Option::Some(target_filter) => target_filter,
        ::core::option::Option::None => return true,
    };

    match filter {
        "rust" | "rs" => file_name.ends_with(".rs"),
        "typescript" | "ts" => file_name.ends_with(".ts") || file_name.ends_with(".tsx"),
        "tsx" => file_name.ends_with(".tsx"),
        "javascript" | "js" => file_name.ends_with(".js") || file_name.ends_with(".jsx"),
        "python" | "py" => file_name.ends_with(".py"),
        "json" => file_name.ends_with(".json"),
        "toml" => file_name.ends_with(".toml"),
        "markdown" | "md" => file_name.ends_with(".md"),
        _ => file_name.ends_with(filter),
    }
}
