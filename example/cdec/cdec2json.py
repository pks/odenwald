#!/usr/bin/env python2

import cdec
import sys, argparse
import json
import codecs

def hg2json(hg, weights):
  """
  output a JSON representation of a cdec hypegraph
  (see http://aclweb.org/aclwiki/index.php?title=Hypergraph_Format )
  """
  res = ''
  res += "{\n"
  res += '"weights":{'+"\n"
  a = []
  for i in weights:
    a.append( '%s:%f'%(json.dumps(i[0]), i[1]) )
  res += ", ".join(a)+"\n"
  res += "},\n"
  res += '"nodes":'+"\n"
  res += "[\n"
  a = []
  a.append( '{ "id":-1, "cat":"root", "span":[-1,-1] }' )
  for i in hg.nodes:
    a.append( '{ "id":%d, "cat":"%s", "span":[%d, %d] }'%(i.id, i.cat, i.span[0], i.span[1]) )
  res += ",\n".join(a)+"\n"
  res += "],\n"
  res += '"edges":'+"\n"
  res += "[\n"
  a = []
  for i in hg.edges:
    s = "{"
    s += '"head":%d'%(i.head_node.id)
    xs = ' "f":{'
    b = []
    for j in i.feature_values:
      b.append( '"%s":%s'%(j[0], j[1]) )
    xs += ", ".join(b)
    xs += "},"
    c = []
    for j in i.tail_nodes:
      c.append(str(j.id))
    if len(c) > 0:
      s += ', "tails":[ %s ],'%(",".join(c))
    else:
      s += ', "tails":[ -1 ],'
    s += xs
    f =  []
    for x in i.trule.f:
      if type(x) == type(u'x'):
        f.append(codecs.encode(x, 'utf-8'))
      else:
        f.append(str(x))
    e = []
    for x in i.trule.e:
      if type(x) == type(u'x'):
        e.append(codecs.encode(x, 'utf-8'))
      else:
        e.append(str(x))
    s += " \"rule\":\"%s ||| %s ||| %s\""%(str(i.trule.lhs), json.dumps(" ".join(f))[1:-1], json.dumps(" ".join(e))[1:-1])
    s += ' }'
    a.append(s)
  res += ",\n".join(a)+"\n"
  res += "]\n"
  res += "}\n"
  return res

def main():
  parser = argparse.ArgumentParser(description='get a proper json representation of cdec hypergraphs')
  parser.add_argument('-c', '--config', required=True, help='decoder configuration')
  parser.add_argument('-w', '--weights', required=True, help='feature weights')
  args = parser.parse_args()
  with open(args.config) as config:
    config = config.read()
  decoder = cdec.Decoder(config)
  decoder.read_weights(args.weights)
  ins = sys.stdin.readline().strip()
  hg = decoder.translate(ins)

  sys.stderr.write( "input:\n '%s'\n"%(ins) )
  sys.stderr.write( "viterbi translation:\n '%s'\n"%(hg.viterbi()) )
  num_nodes = 0
  for i in hg.nodes: num_nodes+=1
  sys.stderr.write( "# nodes = %s\n"%(num_nodes) )
  num_edges = 0
  for i in hg.edges: num_edges+=1
  sys.stderr.write( "# edges = %s\n"%(num_edges) )
  sys.stderr.write( "viterbi score = %s\n"%(round(hg.viterbi_features().dot(decoder.weights), 2)) )

  print hg2json(hg, decoder.weights)

if __name__=="__main__":
  main()

