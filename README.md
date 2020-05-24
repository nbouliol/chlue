# chlue

Basic commands for Philips hue from the command line

## install

```
cargo install chlue
```

## run

A hue user is required to access hue api.

If you have a hue user you can pass it with the `-u` option

If you dont have one, press the bridge button and a user will be created

```
chlue --help

chlue -u hue_username --list-scenes // list all the scenes

chlue -u hue_username -s // select a scene and turn it on / off

chlue -u hue_username -l // select a light and turn it on / off
```
