run:
	cargo run src/foo.txt
logo.png: logo.svg
	convert logo.svg -resize 512x512 logo.png
test2GB.bin:
	dd if=/dev/urandom of=test2GB.bin bs=128M count=16 iflag=fullblock
test:
	cargo test
clean:
	cargo clean
	rm -f test2GB.bin
all:
	cargo build
