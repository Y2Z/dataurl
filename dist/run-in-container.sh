#!/bin/sh

DOCKER=docker
PROG_NAME=dataurl

if which podman 2>&1 > /dev/null; then
    DOCKER=podman
fi

$DOCKER run --rm Y2Z/$PROG_NAME "$@"
