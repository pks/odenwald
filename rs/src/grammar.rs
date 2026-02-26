use std::fmt;
use std::fs;
use std::io::{self, BufRead};

use crate::sparse_vector::SparseVector;

#[derive(Debug, Clone, PartialEq)]
pub enum Symbol {
    T(String),
    NT { symbol: String, index: i32 },
}

impl Symbol {
    pub fn parse(s: &str) -> Symbol {
        let s = s.trim();
        if s.starts_with('[') && s.ends_with(']') {
            let inner = &s[1..s.len() - 1];
            if let Some((sym, idx)) = inner.split_once(',') {
                Symbol::NT {
                    symbol: sym.trim().to_string(),
                    index: idx.trim().parse::<i32>().unwrap() - 1,
                }
            } else {
                Symbol::NT {
                    symbol: inner.trim().to_string(),
                    index: -1,
                }
            }
        } else {
            Symbol::T(s.to_string())
        }
    }

    pub fn is_nt(&self) -> bool {
        matches!(self, Symbol::NT { .. })
    }

    pub fn is_t(&self) -> bool {
        matches!(self, Symbol::T(_))
    }

    pub fn word(&self) -> &str {
        match self {
            Symbol::T(w) => w,
            Symbol::NT { .. } => panic!("word() called on NT"),
        }
    }

    pub fn nt_symbol(&self) -> &str {
        match self {
            Symbol::NT { symbol, .. } => symbol,
            Symbol::T(_) => panic!("nt_symbol() called on T"),
        }
    }

    pub fn nt_index(&self) -> i32 {
        match self {
            Symbol::NT { index, .. } => *index,
            Symbol::T(_) => panic!("nt_index() called on T"),
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Symbol::T(w) => write!(f, "{}", w),
            Symbol::NT { symbol, index } if *index >= 0 => {
                write!(f, "[{},{}]", symbol, index + 1)
            }
            Symbol::NT { symbol, .. } => write!(f, "[{}]", symbol),
        }
    }
}

fn splitpipe(s: &str, n: usize) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut rest = s;
    for _ in 0..n {
        if let Some(pos) = rest.find("|||") {
            parts.push(rest[..pos].trim());
            rest = rest[pos + 3..].trim();
        } else {
            break;
        }
    }
    parts.push(rest.trim());
    parts
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub lhs: Symbol,
    pub rhs: Vec<Symbol>,
    pub target: Vec<Symbol>,
    pub map: Vec<i32>,
    pub f: SparseVector,
    pub arity: usize,
}

impl Rule {
    pub fn new(
        lhs: Symbol,
        rhs: Vec<Symbol>,
        target: Vec<Symbol>,
        map: Vec<i32>,
        f: SparseVector,
    ) -> Self {
        let arity = rhs.iter().filter(|s| s.is_nt()).count();
        Self {
            lhs,
            rhs,
            target,
            map,
            f,
            arity,
        }
    }

    pub fn from_str(s: &str) -> Self {
        let parts = splitpipe(s, 3);
        let lhs = Symbol::parse(parts[0]);

        let mut map = Vec::new();
        let mut arity = 0;
        let rhs: Vec<Symbol> = parts[1]
            .split_whitespace()
            .map(|tok| {
                let sym = Symbol::parse(tok);
                if let Symbol::NT { index, .. } = &sym {
                    map.push(*index);
                    arity += 1;
                }
                sym
            })
            .collect();

        let target: Vec<Symbol> = parts[2]
            .split_whitespace()
            .map(|tok| Symbol::parse(tok))
            .collect();

        let f = if parts.len() > 3 && !parts[3].is_empty() {
            SparseVector::from_kv(parts[3], '=', ' ')
        } else {
            SparseVector::new()
        };

        Self {
            lhs,
            rhs,
            target,
            map,
            f,
            arity,
        }
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ||| {} ||| {}",
            self.lhs,
            self.rhs
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            self.target
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        )
    }
}

pub struct Grammar {
    pub rules: Vec<Rule>,
    pub start_nt: Vec<usize>,
    pub start_t: Vec<usize>,
    pub flat: Vec<usize>,
    pub start_t_by_word: std::collections::HashMap<String, Vec<usize>>,
    pub flat_by_first_word: std::collections::HashMap<String, Vec<usize>>,
    pub start_nt_by_symbol: std::collections::HashMap<String, Vec<usize>>,
}

impl Grammar {
    pub fn from_file(path: &str) -> io::Result<Self> {
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);
        let mut rules = Vec::new();
        let mut start_nt = Vec::new();
        let mut start_t = Vec::new();
        let mut flat = Vec::new();
        let mut start_t_by_word: std::collections::HashMap<String, Vec<usize>> =
            std::collections::HashMap::new();
        let mut flat_by_first_word: std::collections::HashMap<String, Vec<usize>> =
            std::collections::HashMap::new();
        let mut start_nt_by_symbol: std::collections::HashMap<String, Vec<usize>> =
            std::collections::HashMap::new();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim().to_string();
            if line.is_empty() {
                continue;
            }
            let idx = rules.len();
            let rule = Rule::from_str(&line);
            if rule.rhs.first().map_or(false, |s| s.is_nt()) {
                start_nt_by_symbol
                    .entry(rule.rhs[0].nt_symbol().to_string())
                    .or_default()
                    .push(idx);
                start_nt.push(idx);
            } else if rule.arity == 0 {
                flat_by_first_word
                    .entry(rule.rhs[0].word().to_string())
                    .or_default()
                    .push(idx);
                flat.push(idx);
            } else {
                start_t_by_word
                    .entry(rule.rhs[0].word().to_string())
                    .or_default()
                    .push(idx);
                start_t.push(idx);
            }
            rules.push(rule);
        }

        Ok(Self {
            rules,
            start_nt,
            start_t,
            flat,
            start_t_by_word,
            flat_by_first_word,
            start_nt_by_symbol,
        })
    }

    pub fn add_glue_rules(&mut self) {
        let symbols: Vec<String> = self
            .rules
            .iter()
            .map(|r| r.lhs.nt_symbol().to_string())
            .filter(|s| s != "S")
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let mut once = false;
        for symbol in symbols {
            let idx = self.rules.len();
            self.rules.push(Rule::new(
                Symbol::NT {
                    symbol: "S".to_string(),
                    index: -1,
                },
                vec![Symbol::NT {
                    symbol: symbol.clone(),
                    index: 0,
                }],
                vec![Symbol::NT {
                    symbol: symbol.clone(),
                    index: 0,
                }],
                vec![0],
                SparseVector::new(),
            ));
            self.start_nt.push(idx);
            self.start_nt_by_symbol
                .entry(symbol)
                .or_default()
                .push(idx);
            once = true;
        }

        if once {
            let idx = self.rules.len();
            self.rules.push(Rule::new(
                Symbol::NT {
                    symbol: "S".to_string(),
                    index: -1,
                },
                vec![
                    Symbol::NT {
                        symbol: "S".to_string(),
                        index: 0,
                    },
                    Symbol::NT {
                        symbol: "X".to_string(),
                        index: 1,
                    },
                ],
                vec![
                    Symbol::NT {
                        symbol: "S".to_string(),
                        index: 0,
                    },
                    Symbol::NT {
                        symbol: "X".to_string(),
                        index: 1,
                    },
                ],
                vec![0, 1],
                SparseVector::new(),
            ));
            self.start_nt.push(idx);
            self.start_nt_by_symbol
                .entry("S".to_string())
                .or_default()
                .push(idx);
        }
    }

    pub fn add_pass_through_rules(&mut self, words: &[String]) {
        for word in words {
            let idx = self.rules.len();
            self.rules.push(Rule::new(
                Symbol::NT {
                    symbol: "X".to_string(),
                    index: -1,
                },
                vec![Symbol::T(word.clone())],
                vec![Symbol::T(word.clone())],
                vec![],
                SparseVector::new(),
            ));
            self.flat.push(idx);
            self.flat_by_first_word
                .entry(word.clone())
                .or_default()
                .push(idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_parse_t() {
        let s = Symbol::parse("hello");
        assert_eq!(s, Symbol::T("hello".to_string()));
    }

    #[test]
    fn test_symbol_parse_nt() {
        let s = Symbol::parse("[NP,1]");
        assert_eq!(
            s,
            Symbol::NT {
                symbol: "NP".to_string(),
                index: 0
            }
        );
    }

    #[test]
    fn test_symbol_parse_nt_no_index() {
        let s = Symbol::parse("[S]");
        assert_eq!(
            s,
            Symbol::NT {
                symbol: "S".to_string(),
                index: -1
            }
        );
    }

    #[test]
    fn test_rule_from_str() {
        let r = Rule::from_str("[NP] ||| ein [NN,1] ||| a [NN,1] ||| logp=0 use_a=1.0");
        assert_eq!(r.lhs.nt_symbol(), "NP");
        assert_eq!(r.rhs.len(), 2);
        assert_eq!(r.target.len(), 2);
        assert_eq!(r.map, vec![0]);
        assert_eq!(r.arity, 1);
        assert_eq!(*r.f.map.get("use_a").unwrap(), 1.0);
    }

    #[test]
    fn test_rule_display() {
        let r = Rule::from_str("[S] ||| [NP,1] [VP,2] ||| [NP,1] [VP,2] ||| logp=0");
        assert_eq!(r.to_string(), "[S] ||| [NP,1] [VP,2] ||| [NP,1] [VP,2]");
    }
}
