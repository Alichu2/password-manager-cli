# password-manager-cli
A password manager that generates, encrypts, saves and creates backups of passwords all from the terminal.

## Installation
Download the souce code and run:
```cargo build --release```

This will output a binary in target/release/ called password-manager. The binary can be used in the terminal.

## Usage
To generate a password, type the following command:
```password-manager generate```
If you want a longer or shorter password, use the -l flag to specify the number of characters:
```password-manager generate -l 12```

## Contributing:
Feel free to contribute what  ever you want. Please keep in mind this project's goals:
- Simplicity
- Lightwheightness

GUI is no currently necessary.