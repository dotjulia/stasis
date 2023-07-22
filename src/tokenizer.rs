use crate::str_ext::SplitKeepingDelimiterExt;

pub struct Tokenizer {
    content: Vec<String>,
    pos: usize,
}
impl Tokenizer {
    pub fn new(input: &str) -> Self {
        // let content = input
        //     .split([' '])
        //     .map(|e| {
        //         if e.contains(';') {
        //             e.split_inclusive(';').collect::<Vec<_>>()
        //         } else {
        //             [e].into_iter().collect()
        //         }
        //     })
        //     .flatten()
        //     .map(|e| {
        //         if e.contains(";") {
        //             vec![e.replace(";", ""), ";".to_owned()]
        //         } else {
        //             vec![e.replace(";", "")]
        //         }
        //     })
        //     .flatten()
        //     .collect();
        let content = input
            .replace("\n", " ")
            .split(" ")
            .filter(|i| i.len() > 0)
            .map(|e| e.split_keeping_delimiter(&[';']))
            .flatten()
            .map(Into::into)
            .map(|e: &str| e.split_keeping_delimiter(&['(']))
            .flatten()
            .map(Into::into)
            .map(|e: &str| e.split_keeping_delimiter(&[')']))
            .flatten()
            .map(Into::into)
            .map(|e: &str| e.split_keeping_delimiter(&['{']))
            .flatten()
            .map(Into::into)
            .map(|e: &str| e.split_keeping_delimiter(&['}']))
            .flatten()
            .map(Into::into)
            .map(|e: &str| e.to_owned())
            .collect();
        Self { content, pos: 0 }
    }
    pub fn lookahead_until(&self, until: &[&str], matches: &str) -> bool {
        let mut counter = self.pos;
        let mut curr = self.content[counter].clone();
        while !until.iter().any(|e| *e == curr) && counter < self.content.len() {
            curr = self.content[counter].clone();
            if curr == matches {
                return true;
            }
            counter += 1;
        }
        return false;
    }
    pub fn back(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
        }
    }
    pub fn verify_syntax(&mut self) {
        if self.pos != 0 {
            return;
        }
        for (i, token) in self.content.iter().enumerate() {
            if token == "}" && self.content[i - 1] != ";" {
                println!("Token before '}}' is not a ';'. All statements must end in a ';'");
                panic!();
            }
        }
        self.pos = 0;
    }
    pub fn pos(&self) -> usize {
        self.pos
    }
}

#[derive(Debug, Clone)]
pub enum Token {
    Token(String),
    OpeningBracket,
    ClosingBracket,
    OpeningCodeBlock,
    ClosingCodeBlock,
    EndStatement,
}

impl Iterator for Tokenizer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.content.get(self.pos)?;
        self.pos += 1;
        match token.as_ref() {
            ";" => Some(Token::EndStatement),
            "(" => Some(Token::OpeningBracket),
            ")" => Some(Token::ClosingBracket),
            "{" => Some(Token::OpeningCodeBlock),
            "}" => Some(Token::ClosingCodeBlock),
            _ => Some(Token::Token(token.clone())),
        }
    }
}
