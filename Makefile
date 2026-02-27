COMPILER=clang
CFLAGS=-std=c++11 -O3 -Wall
MSGPACK_C_INCLUDE=-I external/msgpack-c/include
MSGPACK_C=external/msgpack-c/lib/libmsgpack.a $(MSGPACK_C_INCLUDE)
JSON_CPP_INCLUDE=-I external/json-cpp/include

SRC=src
BIN=bin

PRINT_BEGIN = @echo -e "\e[1;31mBuilding $@ ...\e[0m"
PRINT_END = @echo -e "\e[1;32mfinished building $@\e[0m"

###############################################################################
# all
#
all: $(BIN)/ow test

###############################################################################
# ow
#

$(BIN)/ow: $(BIN) $(SRC)/hypergraph.o $(SRC)/odenwald.cc
	$(PRINT_BEGIN)
	$(COMPILER) $(CFLAGS) \
		-lstdc++ \
		-lm \
		$(MSGPACK_C) \
		$(SRC)/hypergraph.o \
		$(SRC)/odenwald.cc \
		-o $(BIN)/ow
	$(PRINT_END)

$(BIN):
	mkdir -p $(BIN)

$(SRC)/hypergraph.o: $(SRC)/hypergraph.cc $(SRC)/hypergraph.hh \
											$(SRC)/grammar.hh \
											$(SRC)/semiring.hh $(SRC)/sparse_vector.hh \
											$(SRC)/types.hh
	$(PRINT_BEGIN)
	$(COMPILER) $(CFLAGS) \
		-g \
		-c \
		$(MSGPACK_C_INCLUDE) \
		$(SRC)/hypergraph.cc \
		-o $(SRC)/hypergraph.o
	$(PRINT_END)

###############################################################################
# util
#
util: $(BIN)/make_pak $(BIN)/read_pak

$(BIN)/make_pak: $(BIN) $(SRC)/make_pak.cc external/json-cpp/single_include/json-cpp.hpp \
					$(SRC)/hypergraph.hh $(SRC)/types.hh
	$(PRINT_BEGIN)
	$(COMPILER) $(CFLAGS) \
		-lstdc++ \
		-lm \
		$(MSGPACK_C) \
		$(JSON_CPP_INCLUDE) \
		$(SRC)/make_pak.cc \
		-o $(BIN)/make_pak
	$(PRINT_END)

$(BIN)/read_pak: $(SRC)/read_pak.cc
	$(PRINT_BEGIN)
	$(COMPILER) $(CFLAGS) \
		-lstdc++ \
		$(MSGPACK_C) \
		$(SRC)/read_pak.cc \
		-o $(BIN)/read_pak
	$(PRINT_END)

###############################################################################
# test
#
test: $(BIN)/test_grammar $(BIN)/test_hypergraph $(BIN)/test_parse $(BIN)/test_sparse_vector

$(BIN)/test_grammar: $(BIN) $(SRC)/test_grammar.cc $(SRC)/grammar.hh
	$(PRINT_BEGIN)
	$(COMPILER) $(CFLAGS) \
		-lstdc++ \
		-lm \
		$(MSGPACK_C) \
		$(SRC)/test_grammar.cc \
		-o $(BIN)/test_grammar
	$(PRINT_END)

$(BIN)/test_hypergraph: $(BIN) $(SRC)/test_hypergraph.cc $(SRC)/hypergraph.o $(SRC)/grammar.hh
	$(PRINT_BEGIN)
	$(COMPILER) $(CFLAGS) \
		-lstdc++ \
		-lm \
		$(MSGPACK_C) \
		$(SRC)/hypergraph.o \
		$(SRC)/test_hypergraph.cc \
		-o $(BIN)/test_hypergraph
	$(PRINT_END)

$(BIN)/test_parse: $(BIN) $(SRC)/test_parse.cc $(SRC)/parse.hh \
						$(SRC)/grammar.hh $(SRC)/util.hh
	$(PRINT_BEGIN)
	$(COMPILER) $(CFLAGS) \
		-lstdc++ \
		-lm \
		$(MSGPACK_C) \
		$(SRC)/test_parse.cc \
		-o $(BIN)/test_parse
	$(PRINT_END)

$(BIN)/test_sparse_vector: $(BIN) $(SRC)/test_sparse_vector.cc $(SRC)/sparse_vector.hh
	$(PRINT_BEGIN)
	$(COMPILER) $(CFLAGS) \
		-lstdc++ \
		-lm \
		$(SRC)/test_sparse_vector.cc \
		-o $(BIN)/test_sparse_vector
	$(PRINT_END)

###############################################################################
# clean
#
clean:
	rm -f $(SRC)/*.o
	rm -f $(BIN)/*

