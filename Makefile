.PHONY: all

all: test
clean: teardown
test: testRust
doc: rustDoc

setup:
	docker compose up -d

teardown:
	docker compose down

testRust: setup
	make -C rust test

rustDoc:
	make -C rust doc
