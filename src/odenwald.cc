#include "hypergraph.hh"
#include <ctime>

int
main(int argc, char** argv)
{
  clock_t begin_total = clock();

  // read hg
  clock_t begin_read = clock();
  Hg::Hypergraph hg;
  G::Vocabulary y;
  G::Grammar g;
  Hg::io::read(hg, g.rules, y, argv[1]);
  //Hg::io::manual(hg, g.rules);
  clock_t end_read = clock();
  double elapsed_secs_read = double(end_read - begin_read) / CLOCKS_PER_SEC;
  cerr << "read " << elapsed_secs_read << " s" << endl;

  // viterbi
  clock_t begin_viterbi = clock();
  Hg::Path p;
  Hg::viterbi_path(hg, p);
  vector<string> s;
  Hg::derive(p, p.back()->head, s);
  for (auto it: s)
    cout << it << " ";
  cout << endl;
  clock_t end_viterbi = clock();
  double elapsed_secs_viterbi = double(end_viterbi - begin_viterbi) / CLOCKS_PER_SEC;
  cerr << "viterbi " << elapsed_secs_viterbi << " s" << endl;

  clock_t end_total = clock();
  double elapsed_secs = double(end_total - begin_total) / CLOCKS_PER_SEC;
  cerr << "total " << elapsed_secs << " s" << endl;

  return 0;
}

