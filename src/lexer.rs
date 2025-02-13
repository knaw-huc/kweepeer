use logos::Logos;

#[derive(Logos, Debug, PartialEq)]
pub enum Term<'a> {
    #[regex(r"\w+", |lex| lex.slice())]
    Singular(&'a str),

    // Or regular expressions.
    #[regex(r#""[^"]+""#, |lex|  {
        let quoted = lex.slice();
        &quoted[1..quoted.len() - 1]
    })]
    Phrase(&'a str),

    #[token("AND")]
    #[token("&&")]
    #[token("OR")]
    #[token("||")]
    #[token("NOT")]
    #[token("!")]
    #[token("(")]
    #[token(")")]
    #[regex(r"[\s\t\n]+")]
    #[regex(r"[\+\-]")]
    #[regex(r"[\^~][0-9\.]+")]
    #[regex(r"\{.*\}")]
    None(&'a str),
}

impl<'a> Term<'a> {
    /// Extract terms from a query. Returns the terms and a query template
    /// where terms are marked with `{{` `}}` for easy substitution later.
    pub fn extract_from_query(query: &'a str) -> (Vec<Term<'a>>, String) {
        let mut query_template = String::new();
        let terms = Term::lexer(query)
            .into_iter()
            .filter_map(|term| {
                if let Ok(x) = &term {
                    match x {
                        Term::Singular(y) | Term::Phrase(y) => {
                            query_template += "{{";
                            query_template += y;
                            query_template += "}}";
                        }
                        Term::None(y) => query_template += y,
                    }
                };
                match term {
                    Err(_) => None,
                    Ok(Term::None(_)) => None,
                    Ok(x) => Some(x),
                }
            })
            .collect();
        (terms, query_template)
    }

    /// Returns the term as a string
    pub fn as_str(&self) -> &'a str {
        match self {
            Self::Singular(s) => s,
            Self::Phrase(s) => s,
            Self::None(..) => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn test001_lexer_singular() {
        let terms = Term::extract_from_query("foo");
        assert_eq!(
            terms,
            (vec!(Term::Singular("foo".into())), "{{foo}}".into())
        )
    }

    #[test]
    pub fn test002_lexer_two() {
        let terms = Term::extract_from_query("foo bar");
        assert_eq!(
            terms,
            (
                vec!(Term::Singular("foo".into()), Term::Singular("bar".into())),
                "{{foo}} {{bar}}".into()
            )
        )
    }

    #[test]
    pub fn test003_lexer_phrase() {
        let terms = Term::extract_from_query("\"foo bar\"");
        assert_eq!(
            terms,
            (vec!(Term::Phrase("foo bar".into())), "{{foo bar}}".into())
        )
    }

    #[test]
    pub fn test004_lexer_mixed1() {
        let terms = Term::extract_from_query("foo \"foo bar\"");
        assert_eq!(
            terms,
            (
                vec!(Term::Singular("foo".into()), Term::Phrase("foo bar".into())),
                "{{foo}} {{foo bar}}".into()
            )
        )
    }

    #[test]
    pub fn test005_lexer_mixed2() {
        let terms = Term::extract_from_query("\"foo bar\" foo");
        assert_eq!(
            terms,
            (
                vec!(Term::Phrase("foo bar".into()), Term::Singular("foo".into())),
                "{{foo bar}} {{foo}}".into()
            )
        )
    }

    #[test]
    pub fn test005_lexer_mixed3() {
        let terms = Term::extract_from_query("\"foo bar\" \"bar foo\"");
        assert_eq!(
            terms,
            (
                vec!(Term::Phrase("foo bar".into()), Term::Phrase("bar foo")),
                "{{foo bar}} {{bar foo}}".into()
            )
        )
    }

    #[test]
    pub fn test006_lexer_operator() {
        let terms = Term::extract_from_query("foo AND bar");
        assert_eq!(
            terms,
            (
                vec!(Term::Singular("foo".into()), Term::Singular("bar".into())),
                "{{foo}} AND {{bar}}".into()
            )
        )
    }

    #[test]
    pub fn test006_lexer_operator2() {
        let terms = Term::extract_from_query("+foo -bar");
        assert_eq!(
            terms,
            (
                vec!(Term::Singular("foo".into()), Term::Singular("bar".into())),
                "+{{foo}} -{{bar}}".into()
            )
        )
    }

    #[test]
    pub fn test007_lexer_operator3() {
        let terms = Term::extract_from_query("foo~0.5 bar^3\"");
        assert_eq!(
            terms,
            (
                vec!(Term::Singular("foo".into()), Term::Singular("bar".into())),
                "{{foo}}~0.5 {{bar}}^3".into()
            )
        )
    }

    #[test]
    pub fn test007_lexer_literals() {
        let terms = Term::extract_from_query("\"foo AND bar!\"");
        assert_eq!(
            terms,
            (
                vec!(Term::Phrase("foo AND bar!".into())),
                "{{foo AND bar!}}".into()
            )
        )
    }
}
