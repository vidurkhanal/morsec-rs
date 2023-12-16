use morsec_rs::{many, prefix, take_both, take_left, take_right, while_parse, MorsecStr, Parser};

type Property = (String, String);

#[derive(Debug)]
struct Section {
    title: String,
    properties: Vec<Property>,
}

fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n'
}

fn main() {
    let file_contents = std::fs::read_to_string("src/bin/example.toml").unwrap();
    let section_name_parser = take_right(
        prefix("[".into()),
        take_left(while_parse(|x: char| x != ']'), prefix("]".into())),
    );
    let property_parser = take_both(
        take_left(
            take_left(
                take_left(
                    take_right(
                        while_parse(|x: char| is_whitespace(x)),
                        while_parse(|x: char| !is_whitespace(x) && x != '='),
                    ),
                    while_parse(|x: char| is_whitespace(x)),
                ),
                prefix("=".into()),
            ),
            while_parse(|x: char| is_whitespace(x)),
        ),
        take_left(
            while_parse(|x: char| !is_whitespace(x) && x != '='),
            while_parse(|x: char| is_whitespace(x)),
        ),
    );
    let section_parser = morsec_rs::map(
        take_both(section_name_parser, many(property_parser)),
        |(title, properties)| Section { title, properties },
    );
    let toml_parser = many(section_parser);
    let parsed_toml = toml_parser.parse(MorsecStr::from(file_contents));
    println!("{:?}", parsed_toml);
}
