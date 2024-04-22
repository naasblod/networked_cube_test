# Networked Cube Test

This repo is to demonstrate a jitter problem with physics. In this setup, the client lets the server know that it has finished loading (imagine a situation where a client has connected to a server, but they exchange map data before finally spawning in). Upon this notification message, the server spawns the player in. I am seeking a setup where the server is mainly authoritative which is different from the examples where the player spawns themselves.

## Running the example

To start (listen) server which is both a server and client:

`cargo run -- -l`

To start subsequent clients:

`cargo run -- -c 5678`

Use `-c` flag to set client_id. Default is 1234.

## Issue Description
The physics seems to work fine and runs on both client and server. E.g. you can see other players smoothly since they are interpolated.

But the player's self view is stuttery and I can't figure out whether it is the rollback interrupting the physics or some other problem.
