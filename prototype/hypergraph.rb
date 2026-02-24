require 'zipf'
require_relative 'grammar'

module HG

class HG::Node
  attr_accessor :id, :symbol, :left, :right, :outgoing, :incoming, :score

  def initialize id=nil, symbol='', span=[-1,-1], outgoing=[], incoming=[], score=nil
    @id       = id
    @symbol   = symbol
    @left     = span[0]
    @right    = span[1]
    @outgoing = outgoing
    @incoming = incoming
    @score    = score
  end

  def to_s
    "Node<id=#{@id}, symbol='#{symbol}', span=(#{@left},#{@right}), outgoing:#{@outgoing.size}, incoming:#{@incoming.size}>"
  end
end

class HG::Hypergraph
  attr_accessor :nodes, :edges, :nodes_by_id

  def initialize nodes=[], edges=[], nodes_by_id={}
    @nodes       = nodes
    @edges       = edges
    @nodes_by_id = nodes_by_id
    @arity_      = nil
  end

  def arity
    @arity_ = @edges.map { |e| e.arity }.max if !@arity_
    return @arity_
  end

  def reset
    @edges.each { |e| e.mark = 0 }
  end

  def to_s
    "Hypergraph<nodes:#{@nodes.size}, edges:#{@edges.size}, arity=#{arity}>"
  end

  def to_json weights=nil
    json_s = "{\n"
    json_s += "\"weights\":#{(weights ? weights.to_json : '{}')},\n"
    json_s += "\"nodes\":\n"
    json_s += "[\n"
    json_s += @nodes.map { |n|
      "{ \"id\":#{n.id}, \"cat\":\"#{n.symbol.to_json.slice(1..-1).chomp('"')}\", \"span\":[#{n.left},#{n.right}] }"
    }.join ",\n"
    json_s += "\n],\n"
    json_s += "\"edges\":\n"
    json_s += "[\n"
    json_s += @edges.map { |e|
      "{ \"head\":#{e.head.id}, \"rule\":#{e.rule.to_json}, \"tails\":#{e.tails.map{ |n| n.id }}, \"f\":#{(e.f ? e.f.to_json : '{}')} }"
    }.join ",\n"
    json_s += "\n]\n"
    json_s += "}\n"

    return json_s

  end
end

class HG::Hyperedge
  attr_accessor :head, :tails, :score, :f, :mark, :rule

  def initialize head=Node.new, tails=[], score=0.0, f=SparseVector.new, rule=nil
    @head   = head
    @tails  = tails
    @score  = score
    @f      = f
    @mark   = 0
    @rule   = (rule.class==String ? Grammar::Rule.from_s(rule) : rule)
  end

  def arity
    return @tails.size
  end

  def marked?
    arity == @mark
  end

  def to_s
    "Hyperedge<head=#{@head.id}, rule:'#{@rule.to_s}', tails=#{@tails.map{|n|n.id}}, arity=#{arity}, score=#{@score}, f=#{f.to_s}, mark=#{@mark}>"
  end
end

def HG::topological_sort nodes
  sorted = []
  s = nodes.reject { |n| !n.incoming.empty? }
  while !s.empty?
    sorted << s.shift
    sorted.last.outgoing.each { |e|
      next if e.marked?
      e.mark += 1
      s << e.head if e.head.incoming.reject{ |f| f.marked? }.empty?
    }
  end
  return sorted
end

def HG::init nodes, semiring, root
  nodes.each { |n| n.score=semiring.null }
  root.score = semiring.one
end

def HG::viterbi hypergraph, root, semiring=ViterbiSemiring.new
  toposorted = topological_sort hypergraph.nodes
  init toposorted, semiring, root
  toposorted.each { |n|
    n.incoming.each { |e|
      s = semiring.one
      e.tails.each { |m|
        s = semiring.multiply.call(s, m.score)
      }
      n.score = semiring.add.call(n.score, semiring.multiply.call(s, e.score))
    }
  }
end

def HG::viterbi_path hypergraph, root, semiring=ViterbiSemiring.new
  toposorted = topological_sort hypergraph.nodes
  init toposorted, semiring, root
  best_path = []
  toposorted.each { |n|
    best_edge = nil
    n.incoming.each { |e|
      s = semiring.one
      e.tails.each { |m|
        s = semiring.multiply.call(s, m.score)
      }
      if n.score < semiring.multiply.call(s, e.score) # ViterbiSemiring add
        best_edge = e
      end
      n.score = semiring.add.call(n.score, semiring.multiply.call(s, e.score))
    }
    best_path << best_edge if best_edge
  }

  return best_path, toposorted.last.score
end

def HG::new_obj_path hypergraph, root, semiring=ViterbiSemiring.new
  toposorted = topological_sort hypergraph.nodes
  hypergraph.nodes.each { |n| n.score=1.0/0 }
  root.score = 1
  best_path = []
  toposorted.each { |n|
    best_edge = n.incoming.first
    n.incoming.each { |e|
      s = 0
      e.tails.each { |m|
        s += m.score
      }
      s *= e.score
      if n.score >=  s
        best_edge = e
      end
      n.score = [s, n.score].min
    }
    best_path << best_edge if best_edge
  }

  return best_path, toposorted.last.score
end

def HG::k_best hypergraph, root, semiring=nil
  #TODO
end

def HG::all_paths hypergraph, root
  toposorted = topological_sort hypergraph.nodes
  paths = [[]]
  toposorted.each { |n|
    next if n.incoming.empty?
    new_paths = []
    while !paths.empty?
      p = paths.pop
      n.incoming.each { |e|
        new_paths << p+[e]
      }
    end
    paths = new_paths
  }

  return paths
end

def HG::derive path, cur, carry
  edge = path.select { |e|    e.head.symbol==cur.symbol \
                           && e.head.left==cur.left \
                           && e.head.right==cur.right }.first
  edge.rule.target.each { |i|
    if i.class == Grammar::NT
      derive path, edge.tails[edge.rule.map.index(i.index)], carry
    else
      carry << i
    end
  }
  return carry
end

def HG::read_hypergraph_from_json fn, semiring=RealSemiring.new, log_weights=false, new_obj_model=nil
  nodes = []
  edges = []
  nodes_by_id = {}
  f = ReadFile.new fn
  h = JSON.parse f.read
  w = SparseVector.from_h h['weights']
  h['nodes'].each { |x|
    n = Node.new x['id'], x['symbol'], x['span']
    nodes << n
    nodes_by_id[n.id] = n
  }
  h['edges'].each { |x|
    e = Hyperedge.new(nodes_by_id[x['head']], \
                      x['tails'].map { |j| nodes_by_id[j] }.to_a, \
                      (x['score'] ? semiring.convert.call(x['score'].to_f) : nil), \
                      (x['f'] ? SparseVector.from_h(x['f']) : SparseVector.new), \
                      x['rule'])
    if x['f']
      if log_weights
        e.score = Math.exp(w.dot(e.f))
      elsif new_obj_model
        e.score = e.f.dot(e.f)
        new_obj_model.each { |x|
          wx = x[0]
          wf = x[1]
          e.score -= wx*wf.dot(e.f)
        }
      else
        e.score = w.dot(e.f)
      end
    end
    e.tails.each { |m|
      m.outgoing << e
    }
    e.head.incoming << e
    edges << e
  }
  return Hypergraph.new(nodes, edges), nodes_by_id
end

end #module

