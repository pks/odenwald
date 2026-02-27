use crate::grammar::{Grammar, Symbol};

#[derive(Debug, Clone)]
pub struct Span {
    pub left: i32,
    pub right: i32,
}

#[derive(Debug, Clone)]
pub struct Item {
    pub rule_idx: usize,
    pub left: usize,
    pub right: usize,
    pub dot: usize,
    pub tail_spans: Vec<Option<Span>>,
}

impl Item {
    pub fn from_rule(rule_idx: usize, rhs: &[Symbol], left: usize, right: usize, dot: usize) -> Self {
        let tail_spans = rhs
            .iter()
            .map(|sym| {
                if sym.is_nt() {
                    Some(Span {
                        left: -1,
                        right: -1,
                    })
                } else {
                    None
                }
            })
            .collect();

        Self {
            rule_idx,
            left,
            right,
            dot,
            tail_spans,
        }
    }

    pub fn advance(&self, left: usize, right: usize, dot: usize) -> Self {
        Self {
            rule_idx: self.rule_idx,
            left,
            right,
            dot,
            tail_spans: self.tail_spans.clone(),
        }
    }
}

pub struct Chart {
    _n: usize,
    m: Vec<Vec<Vec<Item>>>,
    b: std::collections::HashSet<(u16, u16, u16)>,
    /// Index: (symbol_id, left) -> sorted vec of right endpoints
    spans_by_left: std::collections::HashMap<(u16, u16), Vec<u16>>,
    sym_to_id: std::collections::HashMap<String, u16>,
    next_sym_id: u16,
}

impl Chart {
    pub fn new(n: usize) -> Self {
        let mut m = Vec::with_capacity(n + 1);
        for _ in 0..=n {
            let mut row = Vec::with_capacity(n + 1);
            for _ in 0..=n {
                row.push(Vec::new());
            }
            m.push(row);
        }
        Self {
            _n: n,
            m,
            b: std::collections::HashSet::new(),
            spans_by_left: std::collections::HashMap::new(),
            sym_to_id: std::collections::HashMap::new(),
            next_sym_id: 0,
        }
    }

    fn sym_id(&mut self, symbol: &str) -> u16 {
        if let Some(&id) = self.sym_to_id.get(symbol) {
            id
        } else {
            let id = self.next_sym_id;
            self.next_sym_id += 1;
            self.sym_to_id.insert(symbol.to_string(), id);
            id
        }
    }

    fn sym_id_lookup(&self, symbol: &str) -> Option<u16> {
        self.sym_to_id.get(symbol).copied()
    }

    pub fn at(&self, i: usize, j: usize) -> &Vec<Item> {
        &self.m[i][j]
    }

    pub fn at_mut(&mut self, i: usize, j: usize) -> &mut Vec<Item> {
        &mut self.m[i][j]
    }

    pub fn add(&mut self, item: Item, i: usize, j: usize, symbol: &str) {
        let sid = self.sym_id(symbol);
        if self.b.insert((i as u16, j as u16, sid)) {
            self.spans_by_left
                .entry((sid, i as u16))
                .or_default()
                .push(j as u16);
        }
        self.m[i][j].push(item);
    }

    pub fn has(&self, symbol: &str, i: usize, j: usize) -> bool {
        if let Some(&sid) = self.sym_to_id.get(symbol) {
            self.b.contains(&(i as u16, j as u16, sid))
        } else {
            false
        }
    }

    /// Returns right endpoints where (symbol, left, right) exists
    pub fn rights_for(&self, symbol: &str, left: usize) -> &[u16] {
        static EMPTY: Vec<u16> = Vec::new();
        if let Some(sid) = self.sym_id_lookup(symbol) {
            if let Some(rights) = self.spans_by_left.get(&(sid, left as u16)) {
                return rights;
            }
        }
        &EMPTY
    }

    /// Check if any entry for (symbol, left, *) exists
    pub fn has_any_at(&self, symbol: &str, left: usize) -> bool {
        if let Some(sid) = self.sym_id_lookup(symbol) {
            self.spans_by_left.contains_key(&(sid, left as u16))
        } else {
            false
        }
    }
}

/// Visit spans bottom-up: from span size `from` up.
/// Yields (left, right) pairs.
pub fn visit<F: FnMut(usize, usize)>(from: usize, l: usize, r: usize, x: usize, mut f: F) {
    for span in from..=(r - x) {
        for k in l..=(r - span) {
            f(k, k + span);
        }
    }
}

pub fn init(input: &[String], n: usize, _active_chart: &mut Chart, passive_chart: &mut Chart, grammar: &Grammar) {
    for i in 0..n {
        if let Some(matching) = grammar.flat_by_first_word.get(&input[i]) {
            for &fi in matching {
                let rule = &grammar.rules[fi];
                let rhs_len = rule.rhs.len();
                if i + rhs_len > n {
                    continue;
                }
                let matches = rule
                    .rhs
                    .iter()
                    .enumerate()
                    .skip(1)
                    .all(|(k, s)| s.word() == input[i + k]);
                if matches {
                    let item = Item::from_rule(fi, &rule.rhs, i, i + rhs_len, rhs_len);
                    passive_chart.add(item, i, i + rhs_len, rule.lhs.nt_symbol());
                }
            }
        }
    }
}

fn scan(item: &mut Item, input: &[String], limit: usize, grammar: &Grammar) -> bool {
    let rhs = &grammar.rules[item.rule_idx].rhs;
    while item.dot < rhs.len() && rhs[item.dot].is_t() {
        if item.right == limit {
            return false;
        }
        if rhs[item.dot].word() == input[item.right] {
            item.dot += 1;
            item.right += 1;
        } else {
            return false;
        }
    }
    true
}

pub fn parse(
    input: &[String],
    n: usize,
    active_chart: &mut Chart,
    passive_chart: &mut Chart,
    grammar: &Grammar,
) {
    visit(1, 0, n, 0, |i, j| {
        // Try to apply rules starting with T
        if let Some(matching) = grammar.start_t_by_word.get(&input[i]) {
            for &ri in matching {
                let mut new_item = Item::from_rule(ri, &grammar.rules[ri].rhs, i, i, 0);
                if scan(&mut new_item, input, j, grammar) {
                    active_chart.at_mut(i, j).push(new_item);
                }
            }
        }

        // Seed active chart with rules starting with NT
        for (symbol, rule_indices) in &grammar.start_nt_by_symbol {
            if !passive_chart.has_any_at(symbol, i) {
                continue;
            }
            for &ri in rule_indices {
                let rule = &grammar.rules[ri];
                if rule.rhs.len() > j - i {
                    continue;
                }
                active_chart
                    .at_mut(i, j)
                    .push(Item::from_rule(ri, &rule.rhs, i, i, 0));
            }
        }

        // Parse
        let mut new_symbols: Vec<String> = Vec::new();
        let mut remaining_items: Vec<Item> = Vec::new();

        while !active_chart.at(i, j).is_empty() {
            let active_item = active_chart.at_mut(i, j).pop().unwrap();
            let mut advanced = false;

            let active_rhs = &grammar.rules[active_item.rule_idx].rhs;
            if active_item.dot < active_rhs.len() && active_rhs[active_item.dot].is_nt() {
                let wanted_symbol = active_rhs[active_item.dot].nt_symbol();
                let k = active_item.right;

                // Use index to directly get matching right endpoints
                let rights: Vec<u16> = passive_chart
                    .rights_for(wanted_symbol, k)
                    .iter()
                    .filter(|&&r| (r as usize) <= j)
                    .copied()
                    .collect();

                for l16 in rights {
                    let l = l16 as usize;

                    let mut new_item =
                        active_item.advance(active_item.left, l, active_item.dot + 1);
                    new_item.tail_spans[new_item.dot - 1] = Some(Span {
                        left: k as i32,
                        right: l as i32,
                    });

                    let new_rhs = &grammar.rules[new_item.rule_idx].rhs;
                    if scan(&mut new_item, input, j, grammar) {
                        if new_item.dot == new_rhs.len() {
                            if new_item.left == i && new_item.right == j {
                                let sym_str = grammar.rules[new_item.rule_idx].lhs.nt_symbol();
                                if !new_symbols.iter().any(|s| s == sym_str) {
                                    new_symbols.push(sym_str.to_string());
                                }
                                passive_chart.add(new_item, i, j, sym_str);
                                advanced = true;
                            }
                        } else if new_item.right + (new_rhs.len() - new_item.dot) <= j {
                            active_chart.at_mut(i, j).push(new_item);
                            advanced = true;
                        }
                    }
                }
            }

            if !advanced {
                remaining_items.push(active_item);
            }
        }

        // Self-filling step
        let mut si = 0;
        while si < new_symbols.len() {
            let s = new_symbols[si].clone();

            // Try start_nt rules from the grammar for this new symbol.
            // This handles rules that weren't seeded in step 2 because
            // the symbol didn't exist in the passive chart at that point.
            if let Some(rule_indices) = grammar.start_nt_by_symbol.get(&s) {
                for &ri in rule_indices {
                    let rule = &grammar.rules[ri];
                    if rule.rhs.len() > j - i {
                        continue;
                    }
                    let mut new_item = Item::from_rule(ri, &rule.rhs, i, i, 0);
                    new_item.dot = 1;
                    new_item.right = j;
                    new_item.tail_spans[0] = Some(Span {
                        left: i as i32,
                        right: j as i32,
                    });
                    if new_item.dot == rule.rhs.len() {
                        let sym_str = rule.lhs.nt_symbol();
                        if !new_symbols.iter().any(|ns| ns == sym_str) {
                            new_symbols.push(sym_str.to_string());
                        }
                        passive_chart.add(new_item, i, j, sym_str);
                    }
                }
            }

            // Also check remaining active items (for rules that were
            // seeded but couldn't advance during the parse loop)
            for item in &remaining_items {
                if item.dot != 0 {
                    continue;
                }
                let item_rhs = &grammar.rules[item.rule_idx].rhs;
                if !item_rhs[item.dot].is_nt() {
                    continue;
                }
                if item_rhs[item.dot].nt_symbol() != s {
                    continue;
                }
                let mut new_item = item.advance(i, j, item.dot + 1);
                new_item.tail_spans[new_item.dot - 1] = Some(Span {
                    left: i as i32,
                    right: j as i32,
                });
                let new_rhs = &grammar.rules[new_item.rule_idx].rhs;
                if new_item.dot == new_rhs.len() {
                    let sym_str = grammar.rules[new_item.rule_idx].lhs.nt_symbol();
                    if !new_symbols.iter().any(|ns| ns == sym_str) {
                        new_symbols.push(sym_str.to_string());
                    }
                    passive_chart.add(new_item, i, j, sym_str);
                }
            }
            si += 1;
        }
    });
}
