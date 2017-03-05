#!/bin/sh

## Written in 2017 by @vflyson
## Released into public domain

# constants
LIMIT=500000 # 500 kB

# options
NAMES=false # print filenames
FULL=false # print full paths for filenames
NOBIG=true # ignore the file limit

# counters
COUNT=0
I=0

# process option flags
for ARG in "$@"
do
    if [ "$ARG" = "-n" ] || [ "$ARG" = "--names" ]; then
        NAMES=true
    elif [ "$ARG" = "-f" ] || [ "$ARG" = "--full-paths" ]; then
        FULL=true
    elif [ "$ARG" = "-l" ] || [ "$ARG" = "--large" ]; then
        NOBIG=false
    fi
done

# count total amount of files to process
for ARG in "$@"
do
    # skip non-regular files and option flags
    if [ ! -f "$ARG" ]; then
        if [ "$ARG" != "-n" ] && [ "$ARG" != "--names" ] \
        && [ "$ARG" != "-f" ] && [ "$ARG" != "--full-paths" ] \
        && [ "$ARG" != "-l" ] && [ "$ARG" != "--large" ]; then
            (>&2 echo "skipping $ARG: not a regular file.")
        fi
        continue
    fi

    # increase the total count of files to process
    COUNT=$((COUNT+1))
done

# main loop
for ARG in "$@"
do
    # skip non-regular files and option flags
    if [ ! -f "$ARG" ]; then
        continue
    fi

    # check the file size
    if $NOBIG; then
        FILESIZE=$(stat -c%s "$ARG")
        if [ $FILESIZE -gt $LIMIT ]; then
            (>&2 echo "skipping $ARG: the file is too big.")
            continue
        fi
    fi

    # detect the mime-type
    MIMETYPE=$(file -b --mime-type "$ARG")

    # printf the file name/path
    if $NAMES; then
        if $FULL; then
            echo "$(tput setaf 1)$ARG$(tput sgr0):"
        else
            echo $(tput setaf 1)$(basename "$ARG")"$(tput sgr0):"
        fi
    fi

    # output the data URI for this file
    echo "data:$MIMETYPE;base64,$(base64 -w0 "$ARG")"

    # increase the counter
    I=$((I+1))

    # put an extra newline between multiple data URIs
    if [ $I -gt 0 ] && [ $I -lt $COUNT ]; then
        echo ""
    fi
done
