#pragma once

#include <algorithm>
#include <cassert>
#include <fstream>
#include <functional>
#include <iostream>
#include <iterator>
#include <list>
#include <msgpack.hpp>
#include <msgpack/fbuffer.hpp>
#include <sstream>
#include <string>
#include <unordered_map>
#include <vector>

#include "grammar.hh"
#include "semiring.hh"
#include "sparse_vector.hh"
#include "types.hh"

using namespace std;

namespace Hg {

struct Node;

struct Edge {
             Node* head;
     vector<Node*> tails;
           score_t score;
          G::Rule* rule;
      unsigned int arity = 0;
      unsigned int mark = 0;

  inline bool is_marked() { return mark >= arity; }
  friend ostream& operator<<(ostream& os, const Edge& e);

          size_t head_id_;
  vector<size_t> tails_ids_; // node ids
          size_t rule_id_;

  MSGPACK_DEFINE(head_id_, tails_ids_, rule_id_, score, arity);
};

struct Node {
          size_t id;
          string symbol;
           short left;
           short right;
         score_t score = 0.0;
   vector<Edge*> incoming;
   vector<Edge*> outgoing;
    unsigned int mark = 0;

  inline bool is_marked() { return mark >= incoming.size(); };
  friend ostream& operator<<(ostream& os, const Node& n);

  MSGPACK_DEFINE(id, symbol, left, right, score);
};

struct Hypergraph {
                   list<Node*> nodes;
                 vector<Edge*> edges;
  unordered_map<size_t, Node*> nodes_by_id;
                  unsigned int arity;
};

template<typename Semiring> void
init(const list<Node*>& nodes, const list<Node*>::iterator root, const Semiring& semiring);

void
reset(const list<Node*> nodes, const vector<Edge*> edges);

void
topological_sort(list<Node*>& nodes, const list<Node*>::iterator root);

void
viterbi(Hypergraph& hg);

typedef vector<Edge*> Path;

void
viterbi_path(Hypergraph& hg, Path& p);

void
sv_path(Hypergraph& hg, Path& p);

void
derive(const Path& p, const Node* cur, vector<string>& carry);

namespace io {

void
read(Hypergraph& hg, vector<G::Rule*>& rules, G::Vocabulary& vocab, const string& fn); // FIXME

void
write(Hypergraph& hg, vector<G::Rule*>& rules, const string& fn); // FIXME

void
manual(Hypergraph& hg, vector<G::Rule*>& rules, G::Vocabulary& vocab);

} // namespace

} // namespace

