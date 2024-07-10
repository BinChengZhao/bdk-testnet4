# BDK Testnet4 Example

This repository demonstrates how to use the BDK (Bitcoin Development Kit) to support Testnet4 by utilizing a custom genesis hash to initialize a wallet, following the guidance provided by the BDK author.

## Prerequisites

- You need to provide an Esplora address that has been set up to index Testnet4 services.
- The example commands below do not explicitly provide the `--esplora-address` parameter because a Testnet4 service is deployed locally at the default address `http://127.0.0.1:3000`. Please ensure you provide the appropriate Esplora address if you are not using a local service.

## Features

- The code includes a built-in public key descriptor, allowing you to check the wallet's balance.
- For operations such as transferring funds, configure the private key descriptor through CLI parameters.

## Usage

### Create Address

To create a new address:

```sh
$ ./target/debug/bdk-testnet4 --esplora-address <your-esplora-address> create-address
```

### Get Balance

To check the wallet balance:

```sh
$ ./target/debug/bdk-testnet4 --esplora-address <your-esplora-address> get-balance 
```

### List Transactions

To list all transactions:

```sh
$ ./target/debug/bdk-testnet4 --esplora-address <your-esplora-address> list-transactions
```

### Send Transaction

To send a transaction (the private key descriptor is hidden in the example, please provide it when you use it):

```sh
$ ./target/debug/bdk-testnet4 --esplora-address <your-esplora-address> pay -r bcrt1qwwf3ckm89aqxzpxhp62ee65s75kn7fnuk0y82g -a 10000
```

### Create Descriptor

To create a descriptor:

```sh
$ ./target/debug/bdk-testnet4 --esplora-address <your-esplora-address> create-descriptor
```

## Notes

- Ensure the Esplora address provided indexes Testnet4 services.
- If you need to perform transactions, include the private key descriptor as a CLI parameter.

This example serves as a starting point for developing applications with BDK on Testnet4. Adjust the commands and parameters as needed to fit your specific use case.