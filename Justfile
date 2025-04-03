web:
    wasm-pack build -d web --target web
    touch web/.nojekyll
    echo "" >> web/.gitignore
    echo "!index.html" >> web/.gitignore
