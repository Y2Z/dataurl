#!/bin/sh

# constants
LIMIT=500000 # 500 KB

# options
NAMES=false # print filenames
FULL=false # print full paths for filenames
NOBIG=false # abide the file limit

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
    elif [ "$ARG" = "-l" ] || [ "$ARG" = "--limit" ]; then
        NOBIG=true
    fi
done

# count total amount of files to process
for ARG in "$@"
do
    # skip non-regular files and option flags
    if [ ! -f "$ARG" ]; then
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
        if [ "$ARG" != "-n" ] && [ "$ARG" != "--names" ] \
        && [ "$ARG" != "-f" ] && [ "$ARG" != "--full-paths" ] \
        && [ "$ARG" != "-l" ] && [ "$ARG" != "--limit" ]; then
            (>&2 echo "skipping $ARG: not a regular file.")
            # put an extra newline between multiple data URIs
            if [ $I -gt 0 ] && [ $I -lt $COUNT ]; then
                (>&2 echo "")
            fi
        fi
        continue
    fi

    # check the file size
    if $NOBIG; then
        FILESIZE=$(stat -c%s "$ARG")
        if [ $FILESIZE -gt $LIMIT ]; then
            (>&2 echo "skipping $ARG: the file is too big.")
            # put an extra newline between multiple data URIs
            if [ $I -gt 0 ] && [ $I -lt $COUNT ]; then
                (>&2 echo "")
            fi
            continue
        fi
    fi

    # detect the mime-type
    MIMETYPE=$(file -b --mime-type "$ARG")

    # printf the file name/path
    if $NAMES; then
        if $FULL; then
            echo "$ARG:"
        else
            echo $(basename "$ARG")":"
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
