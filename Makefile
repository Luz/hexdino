run:
	cargo run src/foo.txt
logo.png: logo.svg
	convert logo.svg -resize 512x512 logo.png
all:
	cargo build
