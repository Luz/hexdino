run:
	cargo run src/foo.txt
logo.png: logo.svg
	convert logo.svg -resize 512x512 logo.png
logo2.png: logo2.svg
	convert logo2.svg -resize 512x512 logo2.png
test:
	cargo test
all:
	cargo build
