//! Parses a hand-written Python type hint like `"collections.abc.Sequence[int] | None"`.
//!
//! The result is a [`PyExpr`] rather than a string so that the names it mentions reach the stub
//! generator, which needs them to emit the matching imports.

use super::{PyConstant, PyExpr};

/// Parses `input` as a Python type hint, or returns a message describing what was wrong with it.
pub fn parse_type_hint(input: &str) -> Result<PyExpr, String> {
    let mut parser = Parser {
        rest: input.trim_start(),
    };
    let expr = parser.union()?;
    if parser.rest.is_empty() {
        Ok(expr)
    } else {
        Err(format!("unexpected `{}`", parser.rest))
    }
}

struct Parser<'a> {
    /// What is left to parse, never starting with whitespace
    rest: &'a str,
}

impl Parser<'_> {
    /// `atom ('|' atom)*`
    fn union(&mut self) -> Result<PyExpr, String> {
        let mut expr = self.postfix()?;
        while self.eat("|") {
            expr = PyExpr::union(expr, self.postfix()?);
        }
        Ok(expr)
    }

    /// An atom followed by any number of `.attr` and `[..]`
    fn postfix(&mut self) -> Result<PyExpr, String> {
        let mut expr = self.atom()?;
        loop {
            if self.eat(".") {
                expr = PyExpr::attribute(expr, self.name()?.to_owned());
            } else if self.eat("[") {
                expr = PyExpr::subscript(expr, self.comma_separated("]")?);
                self.expect("]")?;
            } else {
                return Ok(expr);
            }
        }
    }

    fn atom(&mut self) -> Result<PyExpr, String> {
        if self.eat("...") {
            return Ok(PyExpr::ellipsis());
        }
        if self.eat("[") {
            let elts = match self.comma_separated("]")? {
                PyExpr::Tuple { elts } => elts,
                single => vec![single],
            };
            self.expect("]")?;
            return Ok(PyExpr::List { elts });
        }
        if self.eat("(") {
            let inner = self.union()?;
            self.expect(")")?;
            return Ok(inner);
        }
        if let Some(quote) = ["\"", "'"].into_iter().find(|quote| self.eat(quote)) {
            let end = self
                .rest
                .find(quote)
                .ok_or_else(|| format!("unterminated string, expected a closing {quote}"))?;
            let value = &self.rest[..end];
            self.advance(end + quote.len());
            return Ok(PyExpr::str_constant(value));
        }
        if self.rest.starts_with(|c: char| c.is_ascii_digit()) {
            let end = self
                .rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(self.rest.len());
            let value = self.rest[..end].to_owned();
            self.advance(end);
            return Ok(PyExpr::Constant(PyConstant::Int(value)));
        }
        Ok(match self.name()? {
            "None" => PyExpr::none(),
            "True" => PyExpr::Constant(PyConstant::Bool(true)),
            "False" => PyExpr::Constant(PyConstant::Bool(false)),
            name => PyExpr::builtin(name.to_owned()),
        })
    }

    /// One or more expressions up to `close`, as a tuple when there is more than one
    fn comma_separated(&mut self, close: &str) -> Result<PyExpr, String> {
        let mut elts = vec![self.union()?];
        while self.eat(",") {
            // a trailing comma still makes a tuple, as it does in Python
            if self.rest.starts_with(close) {
                break;
            }
            elts.push(self.union()?);
        }
        Ok(if elts.len() == 1 {
            elts.pop().expect("just checked there is one element")
        } else {
            PyExpr::tuple(elts)
        })
    }

    fn name(&mut self) -> Result<&str, String> {
        let end = self
            .rest
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(self.rest.len());
        if end == 0 || self.rest.starts_with(|c: char| c.is_ascii_digit()) {
            return Err(if self.rest.is_empty() {
                "expected a name, found the end of the type hint".to_owned()
            } else {
                format!("expected a name, found `{}`", self.rest)
            });
        }
        let name = &self.rest[..end];
        self.advance(end);
        Ok(name)
    }

    fn eat(&mut self, token: &str) -> bool {
        let found = self.rest.starts_with(token);
        if found {
            self.advance(token.len());
        }
        found
    }

    fn expect(&mut self, token: &str) -> Result<(), String> {
        if self.eat(token) {
            Ok(())
        } else if self.rest.is_empty() {
            Err(format!(
                "expected `{token}`, found the end of the type hint"
            ))
        } else {
            Err(format!("expected `{token}`, found `{}`", self.rest))
        }
    }

    fn advance(&mut self, bytes: usize) {
        self.rest = self.rest[bytes..].trim_start();
    }
}

#[cfg(test)]
mod tests {
    use super::parse_type_hint;

    #[track_caller]
    fn render(input: &str) -> String {
        format!("{:?}", parse_type_hint(input).unwrap())
    }

    #[test]
    fn round_trips_common_hints() {
        // the `Debug` shape is unwieldy, so check the parses that matter structurally instead
        assert_eq!(render("int"), render(" int "));
        assert_eq!(render("a.b.C[int, str]"), render("a . b . C [ int , str ]"));
        assert_eq!(render("int | None"), render("int|None"));
        assert_eq!(render("tuple[int,]"), render("tuple[int]"));
    }

    #[test]
    fn accepts_the_shapes_a_type_hint_can_take() {
        for hint in [
            "int",
            "None",
            "builtins.int",
            "collections.abc.Generator[typing.Any, typing.Any, int]",
            "list[int] | None",
            "dict[str, list[int]]",
            "collections.abc.Callable[[int, str], bool]",
            "typing.Literal[1, \"a\", True, None]",
            "tuple[int, ...]",
            "(int | str)",
            "MyClass",
        ] {
            assert!(parse_type_hint(hint).is_ok(), "{hint} should parse");
        }
    }

    #[test]
    fn rejects_what_is_not_a_python_expression() {
        // whether the expression is a *usable* type hint is for the type checker reading the
        // generated stub to say; this only has to be able to write it down
        for hint in ["", "list[int", "list[]", "int |", "int int", "a..b", "[,]"] {
            assert!(parse_type_hint(hint).is_err(), "{hint} should not parse");
        }
    }
}
