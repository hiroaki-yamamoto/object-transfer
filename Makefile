.PHONY: all

all: test
clean: teardown
test: testRust

setup:
	docker compose up -d

teardown:
	docker compose stop

testRust: setup
	make -C rust test
