
export INPUT_BOX_ADDRESS=0x59b22D57D4f067708AB0c00552767405926dc768
cast send $INPUT_BOX_ADDRESS "addInput(address,bytes)(bytes32)" 0x70ac08179605AF2D9e75782b8DEcDD3c22aA4D0C 0x68656C6C6F206E6F6465 --mnemonic "test test test test test test test test test test test junk"  --rpc-url "http://localhost:8545"