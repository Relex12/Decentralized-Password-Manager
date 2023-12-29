#!/bin/sh
sudo apt-get -y update
sudo apt-get -y upgrade

sudo apt-get -y install build-essential # Linker C
sudo apt-get -y install pkg-config # Utilisé pour OpenSSL et faire du httpS
sudo apt-get -y install libssl-dev # Idem
sudo apt-get -y install cmake      # Tjrs libC
sudo apt-get -y install libfontconfig1-dev # Pour l'interface graphique
sudo apt-get -y install clang

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
$HOME/.cargo/bin/rustup update
