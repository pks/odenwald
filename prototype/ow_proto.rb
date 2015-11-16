#!/usr/bin/env ruby

require 'trollop'
require 'xmlsimple'
require_relative 'parse'

def read_grammar fn, add_glue, add_pass_through, input=nil
  STDERR.write "> reading grammar '#{fn}'\n"
  grammar = Grammar::Grammar.new fn
  if add_glue
    STDERR.write ">> adding glue rules\n"
    grammar.add_glue_rules
  end
  if add_pass_through
    STDERR.write ">> adding pass-through rules\n"
    grammar.add_pass_through_rules input
  end
  return grammar
end

def main
  conf = Trollop::options do
    opt :input,            "", :type => :string, :default => '-',    :short => '-i'
    opt :grammar,          "", :type => :string, :required => true,  :short => '-g'
    opt :weights,          "", :type => :string, :required => true,  :short => '-w'
    opt :add_glue,         "", :type => :bool,   :default => false,  :short => '-l'
    opt :add_pass_through, "", :type => :bool,   :default => false,  :short => '-p'
    opt :new_obj,          "", :type => :string, :default => nil,    :short => '-N'
  end

  grammar = nil
  if conf[:grammar]
    grammar = read_grammar conf[:grammar], conf[:add_glue], conf[:add_pass_through]
  end

  sgm_input = false
  if ['sgm', 'xml'].include? conf[:input].split('.')[-1]
    sgm_input = true
  end

  STDERR.write "> reading input from '#{conf[:input]}'\n"
  ReadFile.readlines_strip(conf[:input]).each { |input|

    if sgm_input
      x = XmlSimple.xml_in(input)
      input = x['content'].split
    else
      input = input.split
    end
    n = input.size

    if sgm_input && x['grammar']
      grammar = read_grammar x['grammar'], conf[:add_glue], conf[:add_pass_through], input
    elsif conf[:add_pass_through]
      grammar.add_pass_through_rules input
    end


    STDERR.write "> initializing charts\n"
    passive_chart = Parse::Chart.new n
    active_chart = Parse::Chart.new n
    Parse::init input, n, active_chart, passive_chart, grammar

    STDERR.write "> parsing\n"
    Parse::parse input, n, active_chart, passive_chart, grammar

    weights = SparseVector.from_kv(ReadFile.read(conf[:weights]), ' ', "\n")
    if !weights
      weights = SparseVector.new
    end

    hypergraph = passive_chart.to_hg weights

    path = score = nil
    STDERR.write "> viterbi\n"
    semiring = ViterbiSemiring.new
    path, score = HG::viterbi_path hypergraph, hypergraph.nodes_by_id[-1], semiring

    s = HG::derive path, path.last.head, []
    STDOUT.write "#{s.map { |i| i.word }.join ' '} ||| #{Math.log score}\n"
  }
end

main
