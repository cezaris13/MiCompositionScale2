program-name = composition-scale-2
main-file = main.py

install:
ifeq ($(shell uname ),Linux)
	apt-get install pkg-config libssl-dev libdbus-1-dev
endif
	curl https://sh.rustup.rs -sSf | sh
	cargo build

run:
	cargo run

debug:
	export RUST_LOG=debug && \
	export RUST_BACKTRACE=full && \
	cargo run

add-service:
	cp $(program-name).service /etc/systemd/system/

enable-service:
	systemctl enable $(program-name)
	systemctl start $(program-name)

disable-service:
	systemctl disable $(program-name)