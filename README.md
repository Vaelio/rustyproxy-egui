# Rustyproxy

In an attempt to build an alternative to burp and zap, i made this small app.

The point is to take requests from a database in order to inspect/replay/bruteforce those requests.

If you want to contribute, mp me on [Matrix](https://matrix.to/#/@vaelio:matarch.fr) !

## Getting started

First, run the server:

```bash
git clone https://gitlab.com/r2367/rustyproxy-srv
cd rustyproxy-srv
./run.sh -d /tmp/RPTProject
```

Then, run this GUI:

```bash
git clone https://github.com/vaelio/rustyproxy-egui
cd rustyproxy-egui
cargo run -r
```

## Screenshots
![image](https://user-images.githubusercontent.com/6543163/186920854-caf429bf-72e6-413c-9910-58fc10a7dd79.png)
![image](https://user-images.githubusercontent.com/6543163/186920912-449676d5-dbdf-4421-b9a8-dde84337658a.png)
