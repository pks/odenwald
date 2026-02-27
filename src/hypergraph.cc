#include "hypergraph.hh"

namespace Hg {

template<typename Semiring> void
init(const list<Node*>& nodes, const list<Node*>::iterator root, const Semiring& semiring)
{
  for (const auto it: nodes)
    it->score = semiring.null;
  (**root).score = semiring.one;
}

void
reset(const list<Node*> nodes, const vector<Edge*> edges)
{
  for (const auto it: nodes)
    it->mark = 0;
  for (auto it: edges)
    it->mark = 0;
}

void
topological_sort(list<Node*>& nodes, const list<Node*>::iterator root)
{
  auto p = root;
  auto to = nodes.begin();
  while (to != nodes.end()) {
    if ((**p).is_marked()) {
      for (const auto e: (**p).outgoing) { // explore edges
        e->mark++;
        if (e->is_marked()) {
          e->head->mark++;
        }
      }
    }
    if ((**p).is_marked()) {
      nodes.splice(to, nodes, p);
      to++;
      p = to;
    } else {
      ++p;
    }
  }
}

void
viterbi(Hypergraph& hg)
{
  list<Node*>::iterator root = \
    find_if(hg.nodes.begin(), hg.nodes.end(), \
    [](Node* n) { return n->incoming.size() == 0; });

  Hg::topological_sort(hg.nodes, root);
  Semiring::Viterbi<score_t> semiring;
  Hg::init(hg.nodes, root, semiring);

  for (const auto n: hg.nodes) {
    for (const auto e: n->incoming) {
      score_t s = semiring.one;
      for (const auto m: e->tails) {
        s = semiring.multiply(s, m->score);
      }
      n->score = semiring.add(n->score, semiring.multiply(s, e->score));
    }
  }
}

void
viterbi_path(Hypergraph& hg, Path& p)
{
  list<Node*>::iterator root = \
    find_if(hg.nodes.begin(), hg.nodes.end(), \
    [](Node* n) { return n->incoming.size() == 0; });
  //list<Node*>::iterator root = hg.nodes.begin();

  Hg::reset(hg.nodes, hg.edges);
  Hg::topological_sort(hg.nodes, root);
  Semiring::Viterbi<score_t> semiring;
  Hg::init(hg.nodes, root, semiring);

  for (auto n: hg.nodes) {
    Edge* best_edge = nullptr;
    bool best = false;
    for (auto e: n->incoming) {
      score_t s = semiring.one;
      for (auto m: e->tails) {
        s = semiring.multiply(s, m->score);
      }
      if (n->score < semiring.multiply(s, e->score)) { // find max
        best_edge = e;
        best = true;
      }
      n->score = semiring.add(n->score, semiring.multiply(s, e->score));
    }
    if (best)
      p.push_back(best_edge);
  }
}

void
sv_path(Hypergraph& hg, Path& p)
{
  list<Node*>::iterator root = \
    find_if(hg.nodes.begin(), hg.nodes.end(), \
    [](Node* n) { return n->incoming.size() == 0; });
  //list<Node*>::iterator root = hg.nodes.begin();

  Hg::topological_sort(hg.nodes, root);
  //  ^^^ FIXME do I need to do this when reading from file?

  for (const auto it: hg.nodes)
    it->score = 0;
  (**root).score = 1.0;

  for (auto n: hg.nodes) {
    Edge* best_edge;
    bool best = false;
    for (auto e: n->incoming) {
      e->rule->f->repr(cout);
      cout << endl;
    }
  }
}


void
derive(const Path& p, const Node* cur, vector<string>& carry)
{
  Edge* next;
  for (auto it: p) {
    if (it->head->symbol == cur->symbol &&
        it->head->left == cur->left &&
        it->head->right == cur->right) {
      next = it;
    }
  } // FIXME this is probably not so good

  unsigned j = 1;
  for (auto it: next->rule->target) {
    if (it->type() == G::NON_TERMINAL) {
      derive(p, next->tails[next->rule->order[j]-1], carry);
      j++;
    } else {
      carry.push_back(it->symbol());
    }
  }
}

namespace io {

void
read(Hypergraph& hg, vector<G::Rule*>& rules, G::Vocabulary& vocab, const string& fn)
{
  ifstream ifs(fn);
  size_t i = 0, r, n, e;
  msgpack::unpacker pac;
  while(true) {
    pac.reserve_buffer(32*1024);
    ifs.read(pac.buffer(), pac.buffer_capacity());
    size_t bytes = ifs.gcount();
    pac.buffer_consumed(bytes);
    msgpack::unpacked result;
    while(pac.next(&result)) {
      msgpack::object o = result.get();
      if (i ==  0) {
        o.convert(&r);
      } else  if (i == 1) {
        o.convert(&n);
      } else  if (i == 2) {
        o.convert(&e);
      } else if (i > 2 && i <= r+2) {
        string s;
        o.convert(&s);
        G::Rule* rule = new G::Rule;
        G::Rule::from_s(rule, s, vocab);
        rules.push_back(rule);
      } else if (i > r+2 && i <= r+n+2) {
        Node* n = new Node;
        o.convert(n);
        hg.nodes.push_back(n);
        hg.nodes_by_id[n->id] = n;
      } else if (i > n+2 && i <= r+n+e+2) {
        Edge* e = new Edge;
        e->arity = 0;
        o.convert(e);
        e->head = hg.nodes_by_id[e->head_id_];
        hg.edges.push_back(e);
        hg.nodes_by_id[e->head_id_]->incoming.push_back(e);
        e->arity = 0;
        for (auto it = e->tails_ids_.begin(); it != e->tails_ids_.end(); it++) {
          hg.nodes_by_id[*it]->outgoing.push_back(e);
          e->tails.push_back(hg.nodes_by_id[*it]);
          e->arity++;
        }
        e->rule = rules[e->rule_id_];
      } else {
        assert(false); // ERROR
      }
      i++;
    }
    if (!bytes) break;
  }
}

void
write(Hypergraph& hg, vector<G::Rule*>& rules, const string& fn) // FIXME
{
  FILE* file = fopen(fn.c_str(), "wb");
  msgpack::fbuffer fbuf(file);
  msgpack::pack(fbuf, rules.size());
  msgpack::pack(fbuf, hg.nodes.size());
  msgpack::pack(fbuf, hg.edges.size());
  for (auto it = rules.cbegin(); it != rules.cend(); it++)
    msgpack::pack(fbuf, **it);
  for (auto it = hg.nodes.cbegin(); it != hg.nodes.cend(); it++)
    msgpack::pack(fbuf, **it);
  for (auto it = hg.edges.cbegin(); it != hg.edges.cend(); it++)
    msgpack::pack(fbuf, **it);
  fclose(file);
}

void
manual(Hypergraph& hg, vector<G::Rule*>& rules, G::Vocabulary& vocab)
{
  // nodes
  Node* a = new Node; a->id = 0; a->symbol = "root"; a->left = -1;    a->right = -1; a->mark = 0;
  hg.nodes.push_back(a); hg.nodes_by_id[a->id] = a;
  Node* b = new Node; b->id = 1; b->symbol = "NP";   b->left = 0;     b->right = 1;  b->mark = 0;
  hg.nodes.push_back(b); hg.nodes_by_id[b->id] = b;
  Node* c = new Node; c->id = 2; c->symbol = "V";    c->left = 1;     c->right = 2;  c->mark = 0;
  hg.nodes.push_back(c); hg.nodes_by_id[c->id] = c;
  Node* d = new Node; d->id = 3; d->symbol = "JJ";   d->left = 3;     d->right = 4;  d->mark = 0;
  hg.nodes.push_back(d); hg.nodes_by_id[d->id] = d;
  Node* e = new Node; e->id = 4; e->symbol = "NN";   e->left = 3;     e->right = 5;  e->mark = 0;
  hg.nodes.push_back(e); hg.nodes_by_id[e->id] = e;
  Node* f = new Node; f->id = 5; f->symbol = "NP";   f->left = 2;     f->right = 5;  f->mark = 0;
  hg.nodes.push_back(f); hg.nodes_by_id[f->id] = f;
  Node* g = new Node; g->id = 6; g->symbol = "NP";   g->left = 1;     g->right = 5;  g->mark = 0;
  hg.nodes.push_back(g); hg.nodes_by_id[g->id] = g;
  Node* h = new Node; h->id = 7; h->symbol = "S";    h->left = 0;     h->right = 6;  h->mark = 0;
  hg.nodes.push_back(h); hg.nodes_by_id[h->id] = h;

  // rules
  vector<string> rule_strs;
  rule_strs.push_back("[NP] ||| ich ||| i ||| ||| ");
  rule_strs.push_back("[V] ||| sah ||| saw ||| ||| ");
  rule_strs.push_back("[JJ] ||| kleines ||| small ||| ||| ");
  rule_strs.push_back("[JJ] ||| kleines ||| little ||| ||| ");
  rule_strs.push_back("[NN] ||| kleines haus ||| small house ||| ||| ");
  rule_strs.push_back("[NN] ||| kleines haus ||| little house ||| ||| ");
  rule_strs.push_back("[NN] ||| [JJ,1] haus ||| [JJ,1] shell ||| ||| ");
  rule_strs.push_back("[NN] ||| [JJ,1] haus ||| [JJ,1] house ||| ||| ");
  rule_strs.push_back("[NP] ||| ein [NN,1] ||| a [NN,1] ||| ||| ");
  rule_strs.push_back("[VP] ||| [V,1] [NP,2] ||| [V,1] [NP,2] ||| ||| ");
  rule_strs.push_back("[S] ||| [NP,1] [VP,2] ||| [NP,1] [VP,2] ||| ||| ");

  for (auto it: rule_strs) {
    rules.push_back(new G::Rule(it, vocab));
    rules.back()->f = new Sv::SparseVector<string, score_t>();
  }

  // edges
  Edge* q = new Edge; q->head = hg.nodes_by_id[1]; q->tails.push_back(hg.nodes_by_id[0]); q->score = 0.367879441171;
  q->arity = 1; q->mark = 0;
  hg.edges.push_back(q);
  hg.nodes_by_id[1]->incoming.push_back(q);
  hg.nodes_by_id[0]->outgoing.push_back(q);
  q->rule = rules[0];

  Edge* p = new Edge; p->head = hg.nodes_by_id[2]; p->tails.push_back(hg.nodes_by_id[0]); p->score = 0.606530659713;
  p->arity = 1; p->mark = 0;
  hg.edges.push_back(p);
  hg.nodes_by_id[2]->incoming.push_back(p);
  hg.nodes_by_id[0]->outgoing.push_back(p);
  p->rule = rules[1];

  Edge* r = new Edge; r->head = hg.nodes_by_id[3]; r->tails.push_back(hg.nodes_by_id[0]); r->score = 1.0;
  r->arity = 1; r->mark = 0;
  hg.edges.push_back(r);
  hg.nodes_by_id[3]->incoming.push_back(r);
  hg.nodes_by_id[0]->outgoing.push_back(r);
  r->rule = rules[2];

  Edge* s = new Edge; s->head = hg.nodes_by_id[3]; s->tails.push_back(hg.nodes_by_id[0]); s->score = 1.0;
  s->arity = 1; s->mark = 0;
  hg.edges.push_back(s);
  hg.nodes_by_id[3]->incoming.push_back(s);
  hg.nodes_by_id[0]->outgoing.push_back(s);
  s->rule = rules[3];

  Edge* t = new Edge; t->head = hg.nodes_by_id[4]; t->tails.push_back(hg.nodes_by_id[0]); t->score = 1.0;
  t->arity = 1; t->mark = 0;
  hg.edges.push_back(t);
  hg.nodes_by_id[4]->incoming.push_back(t);
  hg.nodes_by_id[0]->outgoing.push_back(t);
  t->rule = rules[4];

  Edge* u = new Edge; u->head = hg.nodes_by_id[4]; u->tails.push_back(hg.nodes_by_id[0]); u->score = 1.0;
  u->arity = 1; u->mark = 0;
  hg.edges.push_back(u);
  hg.nodes_by_id[4]->incoming.push_back(u);
  hg.nodes_by_id[0]->outgoing.push_back(u);
  u->rule = rules[5];

  Edge* v = new Edge; v->head = hg.nodes_by_id[4]; v->tails.push_back(hg.nodes_by_id[3]); v->score = 1.0;
  v->arity = 1; v->mark = 0;
  hg.edges.push_back(v);
  hg.nodes_by_id[4]->incoming.push_back(v);
  hg.nodes_by_id[3]->outgoing.push_back(v);
  v->rule = rules[6];

  Edge* w = new Edge; w->head = hg.nodes_by_id[4]; w->tails.push_back(hg.nodes_by_id[3]); w->score = 2.71828182846;
  w->arity = 1; w->mark = 0;
  hg.edges.push_back(w);
  hg.nodes_by_id[4]->incoming.push_back(w);
  hg.nodes_by_id[3]->outgoing.push_back(w);
  w->rule = rules[7];

  Edge* x = new Edge; x->head = hg.nodes_by_id[5]; x->tails.push_back(hg.nodes_by_id[4]); x->score = 1.0;
  x->arity = 1; x->mark = 0;
  hg.edges.push_back(x);
  hg.nodes_by_id[5]->incoming.push_back(x);
  hg.nodes_by_id[4]->outgoing.push_back(x);
  x->rule = rules[8];

  Edge* y = new Edge; y->head = hg.nodes_by_id[6]; y->tails.push_back(hg.nodes_by_id[2]); y->tails.push_back(hg.nodes_by_id[5]); y->score = 1.0;
  y->arity = 2; y->mark = 0;
  hg.edges.push_back(y);
  hg.nodes_by_id[6]->incoming.push_back(y);
  hg.nodes_by_id[2]->outgoing.push_back(y);
  hg.nodes_by_id[5]->outgoing.push_back(y);
  y->rule = rules[9];

  Edge* z = new Edge; z->head = hg.nodes_by_id[7]; z->tails.push_back(hg.nodes_by_id[1]); z->tails.push_back(hg.nodes_by_id[6]); z->score = 1.0;
  z->arity = 2; z->mark = 0;
  hg.edges.push_back(z);
  hg.nodes_by_id[7]->incoming.push_back(z);
  hg.nodes_by_id[1]->outgoing.push_back(z);
  hg.nodes_by_id[6]->outgoing.push_back(z);
  z->rule = rules[10];
}

} // namespace Hg::io

/*
 * Hg::Node
 *
 */
ostream&
operator<<(ostream& os, const Node& n)
{
  os << \
    "Node<id=" << n.id << \
    ", symbol='" << n.symbol << "'" << \
    ", span=(" << n.left << "," << n.right << ")" \
    ", score=" << n.score << \
    ", incoming:" << n.incoming.size() << \
    ", outgoing:" << n.outgoing.size() << \
    ", mark=" << n.mark << ">";
  return os;
}

/*
 * Hg::Edge
 *
 */
ostream&
operator<<(ostream& os, const Edge& e)
{
  ostringstream _;
  for (auto it: e.tails) {
    _ << it->id;
    if (it != e.tails.back()) _ << ",";
  }
  os << \
    "Edge<head=" << e.head->id << \
    ", tails=[" << _.str() << "]" \
    ", score=" << e.score << \
    ", rule:'";
  e.rule->escaped(os);
  os << "', f=" << "TODO" << \
    ", arity=" << e.arity << \
    ", mark=" << e.mark << ">";
  return os;
}

} // namespace Hg

