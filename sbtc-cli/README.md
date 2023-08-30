# sBTC CLI
The sBTC CLI allows creating and broadcasting sBTC transactions from the command line. The purpose is to aid experimenting with sBTC and testing it. The resulting transactions can be broadcasted to Bitcoin, or kept in hex format as test vectors.

# Installation
From the root of the `core-eng` repo, run
```
cargo install --path sbtc-cli
```

# Usage example
Create a deposit transaction
```
sbtc deposit --wif <WIF of private key> --recipient ST3RBZ4TZ3EK22SZRKGFZYBCKD7WQ5B8FFRS57TT6 --amount 13370 --dkg-wallet tb1pewpc7x6nnea8clm2vn2d8xvpdwvkhucmfdwmm0p6vk2u5xgmwlzsdx3g6w
```

Create a withdrawal transaction
```
sbtc withdraw --wif <WIF of private key> --recipient tb1q0jtfel9tp54dzud28uspe994rv8gajnxc85n8q --amount 42 --dkg-wallet tb1pewpc7x6nnea8clm2vn2d8xvpdwvkhucmfdwmm0p6vk2u5xgmwlzsdx3g6w --fulfillment-fee 1000
```

Broadcast a transaction
```
sbtc broadcast 01000000000101fb27b9579035b82d145b09f3e7e9d02f4ae077a5b3b3fc3356945bb3a3e411650200000000feffffff0300000000000000001a6a1854323c1a755e17b35c75fb5534190b26228187f05781b2823b05000000000000225120cb838f1b539e7a7c7f6a64d4d399816b996bf31b4b5dbdbc3a6595ca191b77c551401100000000001600147c969cfcab0d2ad171aa3f201c94b51b0e8eca6602473044022023371322ebc0311983374c7db5e1eeb2ecb40955c3917e71c3dd75b5e5a364fe02203641377a086795bf816d2b57c4682410cb2cc7bf21987853e6b7030c8a50b44501210215bd6d522931e602fde924571eb472bc1db953484b29ba6542774ebbf083412337322500
```


# Functionality
This list outlines supported and planned functoinality for the CLI.

- Creating OP_RETURN transactions
  - [X] deposit
  - [X] withdrawal
  - [ ] wallet handoff
- Creating OP_DROP transactions
  - [ ] deposit
  - [ ] withdrawal
  - [ ] wallet handoff
- [X] Broadcast transactions
