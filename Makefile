setup:
	docker compose up -d

testRust: setup
	make -C rust test

teardown: testRust
	docker compose down

.PHONY: teardown
