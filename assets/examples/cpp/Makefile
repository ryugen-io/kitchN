CXX = g++
CXXFLAGS = -I../../../include -L../../../target/release -Wall
LIBS = -lhcore_ffi -lpthread -ldl -lm

# Ensure we can find the shared library at runtime
# NOTE: In a real install, libhcore_ffi.so would be in /usr/lib
RUST_LIB_PATH = ../../../target/release

all: main

main: main.cpp
	$(CXX) $(CXXFLAGS) -o main main.cpp $(LIBS) -Wl,-rpath=$(RUST_LIB_PATH)

clean:
	rm -f main
