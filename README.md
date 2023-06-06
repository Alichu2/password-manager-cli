# password-manager-cli
A password manager that generates, encrypts, saves and creates backups of passwords all from the terminal.

## Installation
### Linux
Navigate to releases and download the latest release. Unzip the contents.

**Optional:** Move the contents into ```/usr/local/bin```.

### Windows & Mac OS
You currently have to clone the repository and build it with ```cargo build --release```. The plan is to create build for both operating systems in the future.

## Usage
### Getting Started
Before starting to use password-manager-cli, you need to crate a save directory and file. D this by running:
```commandline
password-manager --new-key
```
This will also ask you for an access key. This key is used to encrypt and decrypt all your passwords, so make sure it is secure and memorable.

### Generating and Saving
To generate a password, type the following command:
```commandline
password-manager generate
```

The number of characters can be also specified:
```commandline
password-manager generate -l 12
```

To save the generated password, use the ```--save``` flag. This will prompt you with your access key and a name that will be used to retrieve the password from the database.
If you desire that the password is not encrypted, use ```--no-encrypt```.

**Note:** If you want to save a password without generating it, use ```add``` instead.

**Note:** The saving process will replace any password with the same name.

### Deleting a Saved Password
To delete a saved password, you use the following command.
```commandline
password-manager delete -p <name>
```

### Loading Saved Passwords
There are two ways to load saved passwords.

- Using the password's name:
```commandline
password-manager load -p <name>
```

- Or loading all the passwords:
```commandline
password-manager load --all
```

### Backups
#### Create Backup
Backups can be created with all saved passwords. These backup files can be encrypted to ensure secure storage. To create a backup, enter:
```commandline
password-manager backup
```
Use ```--no-encrypt``` to create a non encrypted csv file with the passwords.

#### Restore Backup
To restore all the passwords in a backup file, type:
```commandline
password-manager restore path/to/file.txt
```

**Note:** When restoring from a file, it will replace any saved passwords with the same name.

### More
You can use the following to get more information:
- ```--version``` to get the version.
- ```--help``` to get a basic guide.

## Bugs/Issues/Feature Requests
Please create an issue in the Issues tab above.