# SD Importer

Terminal tools that helps to move the images from the SD to a well-structured folders. 

It should be able to move files to a remote location (NAS, Cloud).

## How to install?

```
make build
make install
```

## How to use?

```
Usage: sd_importer <SD path> <Destination directory>
```

## How will be the well-structured format?

The code will take the `Created At` field from the Metadata of the image or video and use it to create (if not exists) the folder of the year, and inside that forlder will create a folder per day. 

Example:

Given this `ls`:

```
-rwxrwxrwx  1 root  staff  16540928 Aug 20  2021 _RX05023.ARW
-rwxrwxrwx  1 root  staff  16610560 Aug 20  2021 _RX05024.ARW
-rwxrwxrwx  1 root  staff  16692480 Aug 20  2021 _RX05025.ARW
```

The code will create the following path structure:
```
- 2021
    - 2018-08-31
        - _RX05023.ARW
        - _RX05024.ARW
        - _RX05025.ARW
```



