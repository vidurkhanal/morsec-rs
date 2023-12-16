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
    struct WrapParser<T> {
        x: T,
    }

    impl<T: Clone> Parser<T> for WrapParser<T> {
        fn parse(&self, input: MorsecStr) -> Result<(MorsecStr, T), MorsecError> {
            Ok((input, self.x.clone()))
        }
    }

    WrapParser { x }
}

pub fn prefix(p: String) -> impl Parser<String> {
    struct PrefixParser {
        prefix: String,
    }

    impl Parser<String> for PrefixParser {
        fn parse(&self, input: MorsecStr) -> Result<(MorsecStr, String), MorsecError> {
            if input.text.is_empty() {
                return Err(MorsecError {
                    message: "Empty input".to_string(),
                    position: input.position,
                });
            }
            let prefix_len = self.prefix.len();
            // dbg!(&input.text);
            let prefix_slice = &input.text[0..prefix_len];
            if prefix_slice == self.prefix {
                let new_pos = input.position + prefix_len;
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

    PrefixParser { prefix: p }
}

pub fn map<T, U, F>(p: impl Parser<T> + 'static, f: F) -> impl Parser<U>
where
    F: Fn(T) -> U + 'static,
{
    struct MapParser<T, F> {
        p: Box<dyn Parser<T>>,
        f: Box<F>,
    }

    impl<T, U, F> Parser<U> for MapParser<T, F>
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
    MapParser {
        p: Box::new(p),
        f: Box::new(f),
    }
}

pub fn bind<T, U, F>(p: impl Parser<T> + 'static, f: F) -> impl Parser<U>
where
    F: Fn(T) -> Box<dyn Parser<U>>,
{
    struct BindParser<T, F> {
        p: Box<dyn Parser<T>>,
        f: Box<F>,
    }

    impl<T, U, F> Parser<U> for BindParser<T, F>
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
    BindParser {
        p: Box::new(p),
        f: Box::new(f),
    }
}

// pub fn while_parse<F>(predicate: F) -> impl Parser<String>
// where
//     F: Fn(char) -> bool,
// {
//     struct WhileParser<F> {
//         predicate: F,
//     }
//
//     impl<F> Parser<String> for WhileParser<F>
//     where
//         F: Fn(char) -> bool,
//     {
//         fn parse(&self, input: MorsecStr) -> Result<(MorsecStr, String), MorsecError> {
//             let mut i = 0;
//             for c in input.text.chars() {
//                 if !(self.predicate)(c) {
//                     break;
//                 }
//                 i += 1;
//             }
//             let new_input = MorsecStr {
//                 text: input.text[i + 1..].to_string(),
//                 position: input.position + i + 1,
//             };
//             Ok((new_input, input.text[0..i + 1].to_string()))
//         }
//     }
//
//     WhileParser { predicate }
// }

pub fn while_parse<F>(predicate: F) -> impl Parser<String>
where
    F: Fn(char) -> bool,
{
    struct WhileParser<F> {
        predicate: F,
    }

    impl<F> Parser<String> for WhileParser<F>
    where
        F: Fn(char) -> bool,
    {
        fn parse(&self, input: MorsecStr) -> Result<(MorsecStr, String), MorsecError> {
            let mut result = String::new();

            for ch in input.text.chars() {
                if (self.predicate)(ch) {
                    result.push(ch);
                } else {
                    break;
                }
            }

            let rest = MorsecStr {
                text: input.text[result.len()..].into(),
                position: result.len(),
            };
            Ok((rest, result))
        }
    }

    WhileParser { predicate }
}

pub fn take_left<T, U>(p: impl Parser<T> + 'static, q: impl Parser<U> + 'static) -> impl Parser<T> {
    struct TakeLeftParser<T, U> {
        p: Box<dyn Parser<T>>,
        q: Box<dyn Parser<U>>,
    }
    impl<T, U> Parser<T> for TakeLeftParser<T, U> {
        fn parse(&self, input: MorsecStr) -> Result<(MorsecStr, T), MorsecError> {
            match self.p.parse(input) {
                Ok((input1, x)) => match self.q.parse(input1) {
                    Ok((input2, _)) => Ok((input2, x)),
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        }
    }
    TakeLeftParser {
        p: Box::new(p),
        q: Box::new(q),
    }
}

pub fn take_right<T, U>(
    p: impl Parser<T> + 'static,
    q: impl Parser<U> + 'static,
) -> impl Parser<U> {
    struct TakeRightParser<T, U> {
        p: Box<dyn Parser<T>>,
        q: Box<dyn Parser<U>>,
    }

    impl<T, U> Parser<U> for TakeRightParser<T, U> {
        fn parse(&self, input: MorsecStr) -> Result<(MorsecStr, U), MorsecError> {
            match self.p.parse(input) {
                Ok((input1, _)) => self.q.parse(input1),
                Err(e) => Err(e),
            }
        }
    }
    TakeRightParser {
        p: Box::new(p),
        q: Box::new(q),
    }
}

pub fn take_both<T, U>(
    p: impl Parser<T> + 'static,
    q: impl Parser<U> + 'static,
) -> impl Parser<(T, U)> {
    struct TakeBothParser<T, U> {
        p: Box<dyn Parser<T>>,
        q: Box<dyn Parser<U>>,
    }
    impl<T, U> Parser<(T, U)> for TakeBothParser<T, U> {
        fn parse(&self, input: MorsecStr) -> Result<(MorsecStr, (T, U)), MorsecError> {
            match self.p.parse(input) {
                Ok((input1, x)) => match self.q.parse(input1) {
                    Ok((input2, y)) => Ok((input2, (x, y))),
                    Err(e) => Err(e),
                },
                Err(e) => Err(e),
            }
        }
    }
    TakeBothParser {
        p: Box::new(p),
        q: Box::new(q),
    }
}

pub fn take_or<T>(p: impl Parser<T> + 'static, q: impl Parser<T> + 'static) -> impl Parser<T> {
    struct TakeOrParser<T> {
        p: Box<dyn Parser<T>>,
        q: Box<dyn Parser<T>>,
    }
    impl<T> Parser<T> for TakeOrParser<T> {
        fn parse(&self, input: MorsecStr) -> Result<(MorsecStr, T), MorsecError> {
            match self.p.parse(input.clone()) {
                Ok((rest, x)) => Ok((rest, x)),
                Err(e1) => match self.q.parse(input.clone()) {
                    Ok((rest, x)) => Ok((rest, x)),
                    Err(e2) => Err(MorsecError {
                        message: format!("{} or {}", e1.message, e2.message),
                        position: input.position,
                    }),
                },
            }
        }
    }
    TakeOrParser {
        p: Box::new(p),
        q: Box::new(q),
    }
}

pub fn optional<T>(p: impl Parser<T> + 'static) -> impl Parser<Option<T>> {
    struct OptionalParser<T> {
        p: Box<dyn Parser<T>>,
    }
    impl<T> Parser<Option<T>> for OptionalParser<T> {
        fn parse(&self, input: MorsecStr) -> Result<(MorsecStr, Option<T>), MorsecError> {
            match self.p.parse(input.clone()) {
                Ok((rest, x)) => Ok((rest, Some(x))),
                Err(_) => Ok((input, None)),
            }
        }
    }
    OptionalParser { p: Box::new(p) }
}

pub fn many<T>(p: impl Parser<T> + 'static) -> impl Parser<Vec<T>> {
    struct ManyParser<T> {
        p: Box<dyn Parser<T>>,
    }
    impl<T> Parser<Vec<T>> for ManyParser<T> {
        fn parse(&self, input: MorsecStr) -> Result<(MorsecStr, Vec<T>), MorsecError> {
            let mut rest = input;
            let mut result = Vec::new();
            loop {
                match self.p.parse(rest.clone()) {
                    Ok((new_rest, x)) => {
                        rest = new_rest;
                        result.push(x);
                    }
                    Err(_) => break,
                }
            }
            Ok((rest, result))
        }
    }
    ManyParser { p: Box::new(p) }
}
