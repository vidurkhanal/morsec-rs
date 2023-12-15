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

pub fn map<T, U, F>(p: impl Parser<T> + 'static, f: F) -> impl Parser<U>
where
    F: Fn(T) -> U + 'static,
{
    struct Wrapper<T, F> {
        p: Box<dyn Parser<T>>,
        f: Box<F>,
    }

    impl<T, U, F> Parser<U> for Wrapper<T, F>
    where
        F: Fn(T) -> U + 'static,
    {
        fn parse(&self, input: MorsecStr) -> Result<(MorsecStr, U), MorsecError> {
            match self.p.parse(input) {
                Ok((p, t)) => Ok((p, (self.f)(t))),
                Err(e) => Err(e),
            }
        }
    }
    Wrapper {
        p: Box::new(p),
        f: Box::new(f),
    }
}

pub fn bind<T, U, F>(p: impl Parser<T> + 'static, f: F) -> impl Parser<U>
where
    F: Fn(T) -> Box<dyn Parser<U>>,
{
    struct Wrapper<T, F> {
        p: Box<dyn Parser<T>>,
        f: Box<F>,
    }

    impl<T, U, F> Parser<U> for Wrapper<T, F>
    where
        F: Fn(T) -> Box<dyn Parser<U>>,
    {
        fn parse(&self, input: MorsecStr) -> Result<(MorsecStr, U), MorsecError> {
            match self.p.parse(input) {
                Ok((p, t)) => (self.f)(t).parse(p),
                Err(e) => Err(e),
            }
        }
    }
    Wrapper {
        p: Box::new(p),
        f: Box::new(f),
    }
}

pub fn while_parse<F>(predicate: F) -> impl Parser<String>
where
    F: Fn(char) -> bool,
{
    struct Wrapper<F> {
        predicate: F,
    }

    impl<F> Parser<String> for Wrapper<F>
    where
        F: Fn(char) -> bool,
    {
        fn parse(&self, input: MorsecStr) -> Result<(MorsecStr, String), MorsecError> {
            let mut i = 0;
            for c in input.text.chars() {
                if !(self.predicate)(c) {
                    break;
                }
                i += 1;
            }
            let new_input = MorsecStr {
                text: input.text[i..].to_string(),
                position: input.position + i,
            };
            Ok((new_input, input.text[0..i].to_string()))
        }
    }

    Wrapper { predicate }
}
