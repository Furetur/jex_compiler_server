# Jex Compiler Server

Server that compiles and runs Jex code.

## Using

To run Jex source code you should _PUSH_ it to the server.

```http request
POST http://localhost:8080
Content-Type: text/plain

println(1+1)
```

or with CURL
```shell
curl -H "Content-Type: text/plain" -d "println(fact(5))" localhost:8080
```

## Running

You can run the server as a simple Rust binary crate.

The **PORT** environment variable must be set, it determines which port the server will listen on.

For example:

```shell
PORT=8080 cargo run
```


