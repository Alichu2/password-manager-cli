# password-manager-cli
A password manager that generates, encrypts, saves and creates backups of passwords all from the terminal.

## Installation
### Download
Navigate to releases and download the file corresponding to your system. Unzip the contents.

## Usage
### Generating and Saving
To generate a password, type the following command:
```commandline
password-manager generate
```

If you want a longer or shorter password, use the -l flag to specify the number of characters:
```commandline
password-manager generate -l 12
```

To save the generated password, use the ```--save``` flag. This will prompt you with your access key and a name that will be used to retrieve the password from the database.
If you desire that the password is not encrypted, use ```--no-encrypt```.

**Note:** if you want to save a password without generating it, use ```add``` instead.

### Deleting a Saved Password
To delete a saved password, you use the following command
```commandline
password-manager delete -p <name>
```
replacing name with the desired password name.

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

### Backing Up Saved Passwords
This will create a file with all the passwords that you have saved. Password manager is then able to add all the
passwords in the file in case that the passwords where lost. The file will by default be encrypted with a key that you
input at the file's creation. The contents of the file can only be read when inputting the same key.

To generate a backup file use:
```commandline
password-manager backup
```
Use ```--no-encrypt``` to not encrypt the backup file.

### Restoring Passwords from Backup File
The restoring process will take your file and add the passwords it finds in the file. To restore use:
```commandline
password-manager restore -f path/to/backupfile
```

### More
You can use the following to get more information:
- ```--version``` to get the version.
- ```--help``` to get a basic guide.

## Contributing:
Feel free to contribute. I cannot guarantee that I will merge your pull request.