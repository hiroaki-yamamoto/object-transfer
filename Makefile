.PHONY: all

all: test
clean: teardown
test: testRust
doc: rustDoc

setup:
	docker compose up -d --wait --wait-timeout 10

teardown:
	docker compose down

testRust: setup
	make -C rust test

rustDoc:
	make -C rust doc
