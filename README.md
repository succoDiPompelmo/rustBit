# rust_bit

![CI](https://github.com/succoDiPompelmo/rustBit/actions/workflows/rust.yml/badge.svg)
![clippy](https://github.com/succoDiPompelmo/rustBit/actions/workflows/rust-clippy.yml/badge.svg)

Toy project of a BitTorrent client implementation. I was curious to see how hard could be to build a BitTorrent client from scratch.

## Usage

Start the server

```bash
cargo run --release
```

Call the API at **/add/magnet** whith whatever HTTP client you have and fill the data part with your magnet

```bash
curl --location 'localhost:8080/add/magnet' \
--header 'Content-Type: text/plain' \
--data '***'
```

To verify everything is working as expected you can take a look at **test.log** file

## Architecture

I made some architectural decision during my exploration of the BitTorrent protocal that I will summarize below:

- The BitTorrent encoding called **Bencode** is implemented without using any external crate. The reason was simple: it's way more fun this way.
- The orchestration between the different concurrent compoments in this create are handled through an Actor model. I did test other options, but the 
Actor model made really easy to make the code readable and maintanable.
- Only the client side of the protocol is implemented. Less work for me.
- Minimal API interface, basically I only allow to add a magnet to download. The objective was to study and experiment with the protocol, having an APIs was not necessary.

## Missing features

For lack of time and resources I cut some corners during the developement, thus some features are missing. No DHT protocol is present to locate available peers.
Integration testing is missing, but looking at other similar repos I'm not alone. Doing integration tests with the BitTorrent protocol is challenging and require
a not so trivial infrastructure management to fit everything inside the CI.

## Code organization

The code is organized in the following folders:

- **actors**: Collection of all actors that take part in the Actor model and the messages that they exchange each others.
- **bencode**: Custom implementation of Bencode encoding.
- **messages**: Rust structs representing the messages that peers exchange to each other in the protocol.
- **peer**: Utilities to manage the connection between peers and the download of pieces.
- **torrent**: Torrent representation in Rust.
- **tracker**: Utilities to manage the communication with the trackers through UDP or TCP.