#[derive(Debug, Clone)]
pub struct MorsecStr {
    pub text: String,
    pub position: usize,
}

#[derive(Debug, Clone)]
pub struct MorsecError {
    pub message: String,
    pub position: usize,
}

impl MorsecStr {
    pub fn from(text: String) -> MorsecStr {
        MorsecStr { text, position: 0 }
    }
}

pub trait Parser<T> {
    fn parse(&self, input: MorsecStr) -> Result<(MorsecStr, T), MorsecError>;
}

pub fn wrap<T: Clone>(x: T) -> impl Parser<T> {
    struct Wrapper<T> {
        x: T,
    }

    impl<T: Clone> Parser<T> for Wrapper<T> {
        fn parse(&self, input: MorsecStr) -> Result<(MorsecStr, T), MorsecError> {
            Ok((input, self.x.clone()))
        }
    }

    Wrapper { x }
}

pub fn prefix(p: String) -> impl Parser<String> {
    struct Wrapper {
        prefix: String,
    }

    impl Parser<String> for Wrapper {
        fn parse(&self, input: MorsecStr) -> Result<(MorsecStr, String), MorsecError> {
            let prefix_len = self.prefix.len();
            let prefix_slice = &input.text[0..prefix_len];
            if prefix_slice == self.prefix {
                let new_pos = input.text.len() - prefix_len;
                let new_input = MorsecStr {
                    text: input.text[prefix_len..].to_string(),
                    position: new_pos,
                };
                Ok((new_input, self.prefix.clone()))
            } else {
                Err(MorsecError {
                    message: format!(
                        "Expected prefix '{}' but got '{}'",
                        self.prefix, prefix_slice
                    ),
                    position: input.position,
                })
            }
        }
    }

    Wrapper { prefix: p }
}
