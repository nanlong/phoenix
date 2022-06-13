## Using phoenix socket client to connect to the websocket server.

1. `npm install -g esbuild`
2. `RUST_LOG=info cargo run`
3. open the `http://localhost:4001`

esbuild assets/js/app.js --bundle --target=es2017 --outdir=./priv/static/assets


.merge(UserSocket::router())

let mut user_socket = UserSocket::new();

let mut channel = Channel::new("rooms:*");

channel.handle_in("test", async move |msg| {
    println!("{}", msg);
});

channel.handle_in("ping", async move |msg| {
    println!("{}", msg);
});

user_socket.add_channel(channel);