use std::str::FromStr;

#[derive(Debug)]
pub enum TokenizingError {
    EmptyInput,
    UnmatchedQuote,
}

pub struct Tokens(Vec<String>);

pub struct Args(pub Vec<String>);

impl FromStr for Tokens {
    type Err = TokenizingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            Err(TokenizingError::EmptyInput)?;
        }
        let mut tokens = Vec::new();
        let mut buffer = String::new();
        let mut in_quotes = false;

        for char in s.chars() {
            if !in_quotes && char.is_whitespace() {
                let trimmed = buffer.trim();
                if trimmed.is_empty() {
                    continue;
                }

                tokens.push(buffer.clone());
                buffer.clear();
                continue;
            }

            if char == '"' && !in_quotes {
                in_quotes = true;
                continue;
            }

            if char == '"' && in_quotes {
                in_quotes = false;
                tokens.push(buffer.clone());
                buffer.clear();
                continue;
            }

            buffer.push(char);
        }

        if in_quotes {
            Err(TokenizingError::UnmatchedQuote)?;
        }

        if !buffer.is_empty() {
            tokens.push(buffer);
        }

        Ok(Tokens(tokens))
    }
}

impl Tokens {
    pub fn command(&self) -> &str {
        self.0[0].as_str()
    }

    pub fn into_args(&self) -> Args {
        if self.0.len() == 1 {
            Args(Vec::new())
        } else {
            Args(self.0[1..].to_vec())
        }
    }
}

#[cfg(test)]
mod test {
    use crate::tokenizer::Tokens;

    #[test]
    fn test_tokenizing() {
        const INPUT_1: &str = "this is a normal string";
        const INPUT_2: &str = r#"this "has quotes""#;
        const INPUT_3: &str = r#"this "has an unmatched quote"#;
        const INPUT_4: &str = "separated     by     multiple        whitespaces";

        let tokens: Tokens = INPUT_1.parse().unwrap();
        assert_eq!(tokens.0, vec!["this", "is", "a", "normal", "string"]);

        let tokens: Tokens = INPUT_2.parse().unwrap();
        assert_eq!(tokens.0, vec!["this", "has quotes"]);

        let tokens = INPUT_3.parse::<Tokens>();
        assert!(tokens.is_err());

        let tokens: Tokens = INPUT_4.parse().unwrap();
        assert_eq!(tokens.0, vec!["separated", "by", "multiple", "whitespaces"])
    }
}
