module Grammar

class T
  attr_accessor :word

  def initialize word
    @word = word
  end

  def to_s
    @word
  end
end

class NT
  attr_accessor :symbol, :index

  def initialize symbol=nil, index=-1
    @symbol = symbol
    @index  = index
  end

  #FIXME? symbol should not contain [, ] or ",
  #       else we're in trouble
  def from_s s
    @symbol, @index = s.delete('[]').split ','
    @symbol.strip!
    @index = @index.to_i-1
  end

  def self.from_s s
    new = NT.new
    new.from_s s
    return new
  end

  #FIXME? indexed by 1
  def to_s
    return "[#{@symbol},#{@index+1}]" if @index>=0
    return "[#{@symbol}]"
  end
end

class Rule
  attr_accessor :lhs, :rhs, :target, :map, :f, :arity

  def initialize lhs=NT.new, rhs=[], target=[], map=[], f=SparseVector.new, arity=0
    @lhs    = lhs
    @rhs    = rhs
    @target = target
    @map    = map
    @f      = f
    @arity  = arity
  end

  def read_rhs_ s, make_meta=false
    a = []
    s.split.map { |x|
      x.strip!
      if x[0] == '[' && x[x.size-1] == ']'
        a << NT.from_s(x)
        if make_meta
          @map << a.last.index
          @arity += 1
        end
      else
        a << T.new(x)
      end
    }
    return a
  end

  def from_s s
    lhs, rhs, target, f = splitpipe s, 3
    @lhs    = NT.from_s lhs
    @rhs    = read_rhs_ rhs, true
    @target = read_rhs_ target
    @f      = (f ? SparseVector.from_kv(f, '=', ' ') : SparseVector.new)
  end

  def self.from_s s
    r = Rule.new
    r.from_s s
    return r
  end

  def to_s
    "#{@lhs.to_s} ||| #{@rhs.map { |x| x.to_s }.join ' '} ||| #{@target.map { |x| x.to_s }.join ' '}"
  end
end

class Grammar
  attr_accessor :rules, :start_nt, :start_t, :flat

  def initialize fn
    @rules = []; @start_nt = []; @start_t = []; @flat = []
    n = 0
    ReadFile.readlines_strip(fn).each_with_index { |s,i|
      STDERR.write '.'; STDERR.write " #{i+1}\n" if (i+1)%40==0
      n += 1
      @rules << Rule.from_s(s)
      if @rules.last.rhs.first.class == NT
        @start_nt << @rules.last
      else
        if rules.last.arity == 0
          @flat << @rules.last
        else
          @start_t << @rules.last
        end
      end
    }
    STDERR.write " #{n}\n"
  end

  def to_s
    @rules.map { |r| r.to_s }.join "\n"
  end

  def add_glue_rules
    @rules.map { |r| r.lhs.symbol }.select { |s| s != 'S' }.uniq.each { |symbol|
      @rules << Rule.new(NT.new('S'), [NT.new(symbol, 0)], [NT.new(symbol, 0)], [0])
      @start_nt << @rules.last
    }
    @rules << Rule.new(NT.new('S'), [NT.new('S', 0), NT.new('X', 1)], [NT.new('S', 0), NT.new('X', 1)], [0, 1])
    @start_nt << @rules.last
  end

  def add_pass_through_rules a
    return if !a
    a.each { |word|
      @rules << Rule.new(NT.new('X'), [T.new(word)], [T.new(word)])
      @flat << @rules.last
    }
  end
end

end #module

