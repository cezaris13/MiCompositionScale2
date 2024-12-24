program-name = composition-scale-2

build:
	cargo build

run:
	cargo run

debug:
	export RUST_LOG=debug && \
	export RUST_BACKTRACE=full && \
	cargo run

test:
	cargo test

coverage-html:
	cargo llvm-cov --html
	open target/llvm-cov/html/index.html

loc:
	find ./src -name '*.rs' | xargs wc -l

add-service:
	cp $(program-name).service /etc/systemd/system/

enable-service:
	systemctl enable $(program-name)
	systemctl start $(program-name)

disable-service:
	systemctl disable $(program-name)

install:
ifeq ($(shell uname ),Linux)
	apt-get install pkg-config libssl-dev libdbus-1-dev
endif
	curl https://sh.rustup.rs -sSf | sh -y 
	cargo install cargo-edit
	cargo build


upgrade-dependencied:
	cargo upgrade
