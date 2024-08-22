#!/bin/bash

# Nome do binário
BINARY_NAME="cv"

# Compilar o projeto em modo release
echo "Compilando o projeto..."
cargo build --release

# Verificar se a compilação foi bem-sucedida
if [ $? -ne 0 ]; then
    echo "Falha na compilação."
    exit 1
fi

# Mover o binário para /usr/local/bin
echo "Movendo o binário para /usr/local/bin..."
sudo mv target/release/$BINARY_NAME /usr/local/bin/

# Verificar se o movimento foi bem-sucedido
if [ $? -ne 0 ]; then
    echo "Falha ao mover o binário."
    exit 1
fi

echo "Instalação concluída com sucesso."