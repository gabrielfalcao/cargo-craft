#!/usr/bin/env bash

at=$(mktemp)
2>$at cargo craft-exec $@
if [ -d "$at" ]; then
    cd $at
else
    1>&2 echo "ERROR: \"$at\" does not exist"
    exit 101
fi
