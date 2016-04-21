PKG_NAME=mux

all: doc build test

.PHONY: run test build clean
run test build clean:
	cargo $@ $(CARGO_FLAGS)

.PHONY: doc
doc: clean
	cargo doc
	echo '<meta http-equiv="refresh" content="0;url='${PKG_NAME}'/index.html">' > target/doc/index.html

.PHONY: docview
docview: doc
	xdg-open target/doc/index.html

.PHONY: publishdoc
publishdoc: doc
	ghp-import -n target/doc
	git push -f origin gh-pages
